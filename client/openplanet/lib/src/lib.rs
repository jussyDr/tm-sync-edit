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
    future::{poll_fn, Future},
    mem, panic,
    pin::Pin,
    task::Poll,
};

use async_compat::CompatExt;
use futures::{executor::block_on, poll, TryStreamExt};
use game::{
    BackToMainMenuFn, Block, BlockInfo, EditNewMap2Fn, EditorCommon, FidsFolder, ItemModel,
    LoadFidFileFn, ManiaPlanet, Menus, NodRef, PlaceBlockFn, PlaceItemFn,
};
use gamebox::{
    engines::game::map::{Direction, ElemColor},
    Vec3,
};
use process::Process;
use shared::{deserialize, framed_tcp_stream, FramedTcpStream, MapDesc, MapParamsDesc, Mood};
use tokio::net::TcpStream;

#[no_mangle]
extern "system" fn Init(mania_planet: &'static mut ManiaPlanet) -> *mut Context {
    panic::set_hook(Box::new(|panic_info| {
        let _ = native_dialog::MessageDialog::new()
            .set_type(native_dialog::MessageType::Error)
            .set_title("SyncEdit.dll")
            .set_text(&panic_info.to_string())
            .show_alert();
    }));

    let context = Context::new(mania_planet);

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
extern "system" fn Join(context: &mut Context) {
    let context_ref = unsafe { &mut *(context as *mut Context) };

    context.connection_future = Some(Box::pin(connection(context_ref)));
}

type ConnectionFuture = dyn Future<Output = Result<(), Box<dyn Error>>>;

struct Context {
    mania_planet: &'static mut ManiaPlanet,
    connection_future: Option<Pin<Box<ConnectionFuture>>>,
    framed_tcp_stream: Option<FramedTcpStream>,
}

impl Context {
    fn new(mania_planet: &'static mut ManiaPlanet) -> Self {
        Self {
            mania_planet,
            connection_future: None,
            framed_tcp_stream: None,
        }
    }
}

async fn connection(context: &mut Context) -> Result<(), Box<dyn Error>> {
    let tcp_stream = TcpStream::connect("127.0.0.1:8369").compat().await?;

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
        .find(|folder| &*folder.name == "Stadium")
        .unwrap();

    let mut block_infos = HashMap::new();
    let mut item_models = HashMap::new();

    let process = Process::open_current()?;
    let main_module_memory = process.main_module_memory()?;
    let load_fid_file_fn = LoadFidFileFn::find(&main_module_memory).unwrap();

    let place_item_fn = PlaceItemFn::find(&main_module_memory).unwrap();

    let block_info_folder = stadium_folder
        .trees
        .iter_mut()
        .find(|folder| &*folder.name == "GameCtnBlockInfo")
        .unwrap();

    load_all_block_infos(block_info_folder, &mut block_infos, load_fid_file_fn);

    let items_folder = stadium_folder
        .trees
        .iter_mut()
        .find(|folder| &*folder.name == "Items")
        .unwrap();

    load_all_item_models(items_folder, &mut item_models, load_fid_file_fn);

    let _ = native_dialog::MessageDialog::new()
        .set_type(native_dialog::MessageType::Error)
        .set_title("SyncEdit.dll")
        .set_text(&format!("{}", item_models.len()))
        .show_alert();

    let editor_common = get_map_editor(context).unwrap();

    let place_block_fn = PlaceBlockFn::find(&main_module_memory).unwrap();

    editor_common.remove_all();

    let air_mode = mem::replace(&mut editor_common.air_mode, true);

    for block in map_desc.blocks {
        let block_info = block_infos.get(&block.block_info_id).unwrap();

        place_block(
            editor_common,
            block_info,
            block.coord,
            block.dir,
            block.elem_color,
            place_block_fn,
        );
    }

    editor_common.air_mode = air_mode;

    for ghost_block in map_desc.ghost_blocks {
        let block_info = block_infos.get(&ghost_block.block_info_id).unwrap();

        editor_common.place_ghost_block(
            block_info,
            ghost_block.coord,
            ghost_block.dir,
            ghost_block.elem_color,
        );
    }

    for free_block in map_desc.free_blocks {
        let block_info = block_infos.get(&free_block.block_info_id).unwrap();

        editor_common.place_free_block(
            block_info,
            free_block.pos,
            free_block.rotation,
            free_block.elem_color,
        );
    }

    for item in map_desc.items {
        let item_model = item_models.get(&item.item_model_id).unwrap();

        unsafe {
            place_item_fn.call(
                editor_common,
                item_model,
                item.pos,
                item.rotation,
                item.pivot_pos,
                item.elem_color,
                item.anim_offset,
            )
        };
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
                    back_to_main_menu_fn.call(context.mania_planet);
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

fn load_all_block_infos(
    folder: &mut FidsFolder,
    block_infos: &mut HashMap<String, NodRef<BlockInfo>>,
    load_fid_file_fn: LoadFidFileFn,
) {
    for folder in folder.trees.iter_mut() {
        load_all_block_infos(folder, block_infos, load_fid_file_fn);
    }

    for file in folder.leaves.iter_mut() {
        let mut nod = unsafe { load_fid_file_fn.call(file).unwrap() };

        if let Some(block_info) = nod.cast_mut::<BlockInfo>() {
            block_infos.insert(block_info.name.to_owned(), NodRef::clone(block_info));
        }
    }
}

fn load_all_item_models(
    folder: &mut FidsFolder,
    item_models: &mut HashMap<String, NodRef<ItemModel>>,
    load_fid_file_fn: LoadFidFileFn,
) {
    for folder in folder.trees.iter_mut() {
        load_all_item_models(folder, item_models, load_fid_file_fn);
    }

    for file in folder.leaves.iter_mut() {
        let mut nod = unsafe { load_fid_file_fn.call(file).unwrap() };

        if let Some(item_model) = nod.cast_mut::<ItemModel>() {
            item_models.insert(item_model.name.to_owned(), NodRef::clone(item_model));
        }
    }
}

fn place_block(
    editor_common: &mut EditorCommon,
    block_info: &BlockInfo,
    coord: Vec3<u8>,
    dir: Direction,
    elem_color: ElemColor,
    place_block_fn: PlaceBlockFn,
) -> Option<NodRef<Block>> {
    unsafe {
        if editor_common.can_place_block(block_info, coord, dir) {
            place_block_fn.call(editor_common, block_info, coord, dir, elem_color)
        } else {
            None
        }
    }
}
