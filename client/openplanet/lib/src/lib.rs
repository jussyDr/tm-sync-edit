#![warn(clippy::unwrap_used)]

mod process;

/// Interacting with the game.
mod game {
    mod classes;
    mod fns;

    pub use classes::*;
    pub use fns::*;
}

use std::{
    collections::HashMap,
    error::Error,
    ffi::{c_char, CStr},
    fs,
    future::{poll_fn, Future},
    mem,
    net::{IpAddr, SocketAddr},
    panic,
    path::Path,
    pin::Pin,
    str::FromStr,
    task::Poll,
};

use async_compat::CompatExt;
use futures::{executor::block_on, poll, TryStreamExt};
use game::{
    BackToMainMenuFn, Block, BlockInfo, EditNewMap2Fn, EditorCommon, FidsFolder,
    GenerateBlockInfoFn, ItemModel, LoadFidFileFn, ManiaPlanet, Menus, NodRef, PlaceBlockFn,
    PlaceItemFn,
};
use gamebox::{
    engines::game::map::{Direction, ElemColor, YawPitchRoll},
    Vec3,
};
use process::Process;
use shared::{
    deserialize, framed_tcp_stream, hash, FramedTcpStream, Hash, MapDesc, MapParamsDesc, ModelId,
    Mood,
};
use tokio::net::TcpStream;

#[no_mangle]
extern "system" fn Init(
    mania_planet: NodRef<ManiaPlanet>,
    program_data_folder: NodRef<FidsFolder>,
) -> *mut Context {
    panic::set_hook(Box::new(|panic_info| {
        let _ = native_dialog::MessageDialog::new()
            .set_type(native_dialog::MessageType::Error)
            .set_title("SyncEdit.dll")
            .set_text(&panic_info.to_string())
            .show_alert();
    }));

    let context = Context::new(mania_planet, program_data_folder);

    Box::into_raw(Box::new(context))
}

#[no_mangle]
unsafe extern "system" fn Destroy(context: *mut Context) {
    drop(Box::from_raw(context));
}

#[no_mangle]
extern "system" fn Update(context: &mut Context) {
    block_on(async {
        if let Some(connection_future) = &mut context.connection_future {
            if poll!(connection_future).is_ready() {
                context.connection_future = None;
            }
        }
    })
}

#[no_mangle]
extern "system" fn Join(context: &mut Context, host: *const c_char, port: *const c_char) {
    let context_ref = unsafe { &mut *(context as *mut Context) };

    let host = unsafe { CStr::from_ptr(host).to_str().unwrap().to_owned() };
    let port = unsafe { CStr::from_ptr(port).to_str().unwrap().to_owned() };

    context.connection_future = Some(Box::pin(connection(context_ref, host, port)));
}

type ConnectionFuture = dyn Future<Output = Result<(), Box<dyn Error>>>;

struct Context {
    mania_planet: NodRef<ManiaPlanet>,
    program_data_folder: NodRef<FidsFolder>,
    connection_future: Option<Pin<Box<ConnectionFuture>>>,
    framed_tcp_stream: Option<FramedTcpStream>,
}

impl Context {
    fn new(mania_planet: NodRef<ManiaPlanet>, program_data_folder: NodRef<FidsFolder>) -> Self {
        Self {
            mania_planet,
            program_data_folder,
            connection_future: None,
            framed_tcp_stream: None,
        }
    }
}

async fn connection(
    context: &mut Context,
    host: String,
    port: String,
) -> Result<(), Box<dyn Error>> {
    let ip_addr = IpAddr::from_str(&host)?;
    let port = u16::from_str(&port)?;
    let socket_addr = SocketAddr::new(ip_addr, port);

    let tcp_stream = TcpStream::connect(socket_addr).compat().await?;

    let mut framed_tcp_stream = framed_tcp_stream(tcp_stream);

    let frame = framed_tcp_stream.try_next().await?.unwrap();
    let map_params_desc: MapParamsDesc = deserialize(&frame)?;

    open_map_editor(context, map_params_desc).await?;

    let frame = framed_tcp_stream.try_next().await?.unwrap();
    let map_desc: MapDesc = deserialize(&frame)?;

    let game_data_folder = &mut context.mania_planet.fid_file.parent_folder;

    let stadium_folder = game_data_folder
        .trees
        .iter_mut()
        .find(|folder| &*folder.path == "Stadium")
        .unwrap();

    let mut block_infos = HashMap::new();
    let mut item_models = HashMap::new();

    let process = Process::open_current()?;
    let main_module_memory = process.main_module_memory()?;
    let load_fid_file_fn = LoadFidFileFn::find(&main_module_memory).unwrap();

    let place_item_fn = PlaceItemFn::find(&main_module_memory).unwrap();
    let generate_block_info_fn = GenerateBlockInfoFn::find(&main_module_memory).unwrap();

    let block_info_folder = stadium_folder
        .trees
        .iter_mut()
        .find(|folder| &*folder.path == "GameCtnBlockInfo")
        .unwrap();

    load_block_infos(block_info_folder, &mut block_infos, load_fid_file_fn);

    let items_folder = stadium_folder
        .trees
        .iter_mut()
        .find(|folder| &*folder.path == "Items")
        .unwrap();

    load_item_models(items_folder, &mut item_models, load_fid_file_fn);

    let mut custom_block_infos = HashMap::new();
    let mut custom_item_models = HashMap::new();

    load_custom_objects(
        context,
        &map_desc,
        &mut custom_block_infos,
        &mut custom_item_models,
        load_fid_file_fn,
        generate_block_info_fn,
    )
    .unwrap();

    let editor_common = get_map_editor(context).unwrap();

    let place_block_fn = PlaceBlockFn::find(&main_module_memory).unwrap();

    editor_common.remove_all();

    let mut blocks = HashMap::new();
    let mut ghost_blocks = HashMap::new();
    let mut free_blocks = HashMap::new();
    let mut items = HashMap::new();

    let air_mode = mem::replace(&mut editor_common.air_mode, true);

    for block_desc in map_desc.blocks {
        let block_info = match block_desc.block_info_id {
            ModelId::Game { ref id } => block_infos.get(id).unwrap(),
            ModelId::Custom { hash } => custom_block_infos.get(&hash).unwrap(),
        };

        let block = place_block(
            editor_common,
            block_info,
            block_desc.coord,
            block_desc.dir,
            block_desc.is_air_variant,
            block_desc.elem_color,
            place_block_fn,
        );

        if let Some(block) = block {
            blocks.insert(block_desc, block);
        }
    }

    editor_common.air_mode = air_mode;

    for ghost_block_desc in map_desc.ghost_blocks {
        let block_info = match ghost_block_desc.block_info_id {
            ModelId::Game { ref id } => block_infos.get(id).unwrap(),
            ModelId::Custom { hash } => custom_block_infos.get(&hash).unwrap(),
        };

        let ghost_block = editor_common.place_ghost_block(
            block_info,
            ghost_block_desc.coord,
            ghost_block_desc.dir,
            ghost_block_desc.is_air_variant,
            ghost_block_desc.elem_color,
        );

        if let Some(ghost_block) = ghost_block {
            ghost_blocks.insert(ghost_block_desc, ghost_block);
        }
    }

    for free_block_desc in map_desc.free_blocks {
        let block_info = match free_block_desc.block_info_id {
            ModelId::Game { ref id } => block_infos.get(id).unwrap(),
            ModelId::Custom { hash } => custom_block_infos.get(&hash).unwrap(),
        };

        let free_block = editor_common.place_free_block(
            block_info,
            Vec3 {
                x: free_block_desc.position.x.into_inner(),
                y: free_block_desc.position.y.into_inner(),
                z: free_block_desc.position.z.into_inner(),
            },
            YawPitchRoll {
                yaw: free_block_desc.yaw.into_inner(),
                pitch: free_block_desc.pitch.into_inner(),
                roll: free_block_desc.roll.into_inner(),
            },
            free_block_desc.elem_color,
        );

        if let Some(free_block) = free_block {
            free_blocks.insert(free_block_desc, free_block);
        }
    }

    for item_desc in map_desc.items {
        let item_model = match item_desc.item_model_id {
            ModelId::Game { ref id } => item_models.get(id).unwrap(),
            ModelId::Custom { hash } => custom_item_models.get(&hash).unwrap(),
        };

        let item = place_item_fn.call(
            editor_common,
            item_model,
            Vec3 {
                x: item_desc.position.x.into_inner(),
                y: item_desc.position.y.into_inner(),
                z: item_desc.position.z.into_inner(),
            },
            YawPitchRoll {
                yaw: item_desc.yaw.into_inner(),
                pitch: item_desc.pitch.into_inner(),
                roll: item_desc.roll.into_inner(),
            },
            Vec3 {
                x: item_desc.pivot_position.x.into_inner(),
                y: item_desc.pivot_position.y.into_inner(),
                z: item_desc.pivot_position.z.into_inner(),
            },
            item_desc.elem_color,
            item_desc.anim_offset,
        );

        if let Some(item) = item {
            items.insert(item_desc, item);
        }
    }

    while framed_tcp_stream.try_next().await?.is_some() {}

    Ok(())
}

async fn open_map_editor(
    context: &mut Context,
    params: MapParamsDesc,
) -> Result<(), Box<dyn Error>> {
    let process = Process::open_current()?;
    let main_module_memory = process.main_module_memory()?;

    let back_to_main_menu_fn = BackToMainMenuFn::find(&main_module_memory).unwrap();
    let edit_new_map_2_fn = EditNewMap2Fn::find(&main_module_memory).unwrap();

    let future = poll_fn(|_| {
        let module_stack = &context.mania_planet.switcher.module_stack;

        let editor_open = module_stack
            .iter()
            .any(|module| module.is_instance_of::<EditorCommon>());

        if editor_open {
            return Poll::Ready(());
        }

        if let Some(current_module) = module_stack.last() {
            if current_module.is_instance_of::<Menus>() {
                let mood_name = match params.mood {
                    Mood::Day => "Day",
                    Mood::Sunset => "Sunset",
                    Mood::Night => "Night",
                    Mood::Sunrise => "Sunrise",
                };

                let decoration_id = format!("48x48Screen155{mood_name}");

                unsafe {
                    edit_new_map_2_fn.call(
                        &mut context.mania_planet.mania_title_control_script_api,
                        &decoration_id,
                    );
                };
            } else {
                unsafe {
                    back_to_main_menu_fn.call(&mut context.mania_planet);
                }
            }
        }

        Poll::Pending
    });

    future.await;

    Ok(())
}

fn get_map_editor(context: &mut Context) -> Option<&mut NodRef<EditorCommon>> {
    context
        .mania_planet
        .switcher
        .module_stack
        .iter_mut()
        .filter_map(|module| module.cast_mut::<EditorCommon>())
        .next()
}

fn load_block_infos(
    folder: &mut FidsFolder,
    block_infos: &mut HashMap<String, NodRef<BlockInfo>>,
    load_fid_file_fn: LoadFidFileFn,
) -> Result<(), Box<dyn Error>> {
    for folder in folder.trees.iter_mut() {
        load_block_infos(folder, block_infos, load_fid_file_fn)?;
    }

    for file in folder.leaves.iter_mut() {
        let mut nod = load_fid_file_fn
            .call(file)
            .ok_or("Failed to load fid file")?;

        if let Some(block_info) = nod.cast_mut::<BlockInfo>() {
            block_infos.insert(block_info.name.to_owned(), NodRef::clone(block_info));
        }
    }

    Ok(())
}

fn load_item_models(
    folder: &mut FidsFolder,
    item_models: &mut HashMap<String, NodRef<ItemModel>>,
    load_fid_file_fn: LoadFidFileFn,
) -> Result<(), Box<dyn Error>> {
    for folder in folder.trees.iter_mut() {
        load_item_models(folder, item_models, load_fid_file_fn)?;
    }

    for file in folder.leaves.iter_mut() {
        let mut nod = load_fid_file_fn
            .call(file)
            .ok_or("Failed to load fid file")?;

        if let Some(item_model) = nod.cast_mut::<ItemModel>() {
            item_models.insert(item_model.name.to_owned(), NodRef::clone(item_model));
        }
    }

    Ok(())
}

fn load_custom_objects(
    context: &mut Context,
    map_desc: &MapDesc,
    custom_block_infos: &mut HashMap<Hash, NodRef<BlockInfo>>,
    custom_item_models: &mut HashMap<Hash, NodRef<ItemModel>>,
    load_fid_file_fn: LoadFidFileFn,
    generate_block_info_fn: GenerateBlockInfoFn,
) -> Result<(), Box<dyn Error>> {
    let program_data_folder_path = Path::new(&*context.program_data_folder.path);

    let mut sync_edit_folder_path = program_data_folder_path.to_owned();
    sync_edit_folder_path.push("SyncEdit");

    fs::create_dir(&sync_edit_folder_path)?;

    let result = load_custom_objects_using_folder(
        context,
        map_desc,
        custom_block_infos,
        custom_item_models,
        load_fid_file_fn,
        generate_block_info_fn,
        &sync_edit_folder_path,
    );

    fs::remove_dir_all(&sync_edit_folder_path)?;

    result
}

fn load_custom_objects_using_folder(
    context: &mut Context,
    map_desc: &MapDesc,
    custom_block_infos: &mut HashMap<Hash, NodRef<BlockInfo>>,
    custom_item_models: &mut HashMap<Hash, NodRef<ItemModel>>,
    load_fid_file_fn: LoadFidFileFn,
    generate_block_info_fn: GenerateBlockInfoFn,
    folder: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut custom_block_hashes = vec![];

    for custom_block in &map_desc.custom_blocks {
        let hash = hash(&custom_block.bytes);

        let mut file_path = folder.to_owned();
        file_path.push(hash.to_hex().as_str());
        file_path.set_extension("Block.Gbx");

        fs::write(&file_path, &custom_block.bytes)?;

        custom_block_hashes.push(hash);
    }

    let mut custom_item_hashes = vec![];

    for custom_item in &map_desc.custom_items {
        let hash = hash(&custom_item.bytes);

        let mut file_path = folder.to_owned();
        file_path.push(hash.to_hex().as_str());
        file_path.set_extension("Item.Gbx");

        fs::write(&file_path, &custom_item.bytes)?;

        custom_item_hashes.push(hash);
    }

    context.program_data_folder.update_tree(false);

    let sync_edit_folder = context
        .program_data_folder
        .trees
        .iter_mut()
        .find(|folder| &*folder.path == "SyncEdit")
        .unwrap();

    sync_edit_folder.update_tree(false);

    for hash in custom_block_hashes {
        let file_name = format!("{}.Block.Gbx", hash.to_hex());

        let file = sync_edit_folder
            .leaves
            .iter_mut()
            .find(|file| *file.name == *file_name)
            .unwrap();

        let mut nod = load_fid_file_fn
            .call(file)
            .ok_or("Failed to load fid file")?;

        let item_model = nod.cast_mut::<ItemModel>().unwrap();

        generate_block_info_fn.call(item_model);

        let block_info = item_model
            .entity_model
            .as_mut()
            .unwrap()
            .cast_mut::<BlockInfo>()
            .unwrap();

        custom_block_infos.insert(hash, NodRef::clone(block_info));
    }

    for hash in custom_item_hashes {
        let file_name = format!("{}.Item.Gbx", hash.to_hex());

        let file = sync_edit_folder
            .leaves
            .iter_mut()
            .find(|file| *file.name == *file_name)
            .unwrap();

        let mut nod = load_fid_file_fn
            .call(file)
            .ok_or("Failed to load fid file")?;

        let item_model = nod.cast_mut::<ItemModel>().unwrap();

        custom_item_models.insert(hash, NodRef::clone(item_model));
    }

    Ok(())
}

fn place_block(
    editor_common: &mut EditorCommon,
    block_info: &BlockInfo,
    coord: Vec3<u8>,
    dir: Direction,
    is_air_variant: bool,
    elem_color: ElemColor,
    place_block_fn: PlaceBlockFn,
) -> Option<NodRef<Block>> {
    if editor_common.can_place_block(block_info, coord, dir) {
        unsafe {
            place_block_fn.call(
                editor_common,
                block_info,
                coord,
                dir,
                is_air_variant,
                elem_color,
            )
        }
    } else {
        None
    }
}
