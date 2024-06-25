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
    BackToMainMenuFn, Block, BlockInfo, EditNewMap2Fn, EditorCommon, FidsFolder, LoadFidFileFn,
    ManiaPlanet, Menus, NodRef, PlaceBlockFn,
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

    let block_info_folder = stadium_folder
        .trees
        .iter_mut()
        .find(|folder| &*folder.name == "GameCtnBlockInfo")
        .unwrap();

    let mut block_infos: HashMap<String, NodRef<BlockInfo>> = HashMap::new();

    let process = Process::open_current()?;
    let main_module_memory = process.main_module_memory()?;
    let load_fid_file_fn = LoadFidFileFn::find(&main_module_memory).unwrap();

    preload_all_block_infos(block_info_folder, &mut block_infos, load_fid_file_fn);

    let editor_common = get_map_editor(context).unwrap();

    let place_block_fn = PlaceBlockFn::find(&main_module_memory).unwrap();

    let air_mode = mem::replace(&mut editor_common.air_mode, true);

    unsafe { editor_common.remove_all() };

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

    for ghost_block in map_desc.ghost_blocks {
        let block_info = block_infos.get(&ghost_block.block_info_id).unwrap();

        place_ghost_block(
            editor_common,
            block_info,
            ghost_block.coord,
            ghost_block.dir,
            ghost_block.elem_color,
        );
    }

    editor_common.air_mode = air_mode;

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

fn preload_all_block_infos(
    folder: &mut FidsFolder,
    block_infos: &mut HashMap<String, NodRef<BlockInfo>>,
    load_fid_file_fn: LoadFidFileFn,
) {
    for folder in folder.trees.iter_mut() {
        preload_all_block_infos(folder, block_infos, load_fid_file_fn);
    }

    for file in folder.leaves.iter_mut() {
        let mut nod = unsafe { load_fid_file_fn.call(file).unwrap() };

        if let Some(block_info) = nod.cast_mut::<BlockInfo>() {
            block_infos.insert(block_info.name.to_owned(), NodRef::clone(block_info));
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

fn place_ghost_block(
    editor_common: &mut EditorCommon,
    block_info: &BlockInfo,
    coord: Vec3<u8>,
    dir: Direction,
    elem_color: ElemColor,
) -> Option<NodRef<Block>> {
    unsafe {
        if editor_common.can_place_block(block_info, coord, dir) {
            editor_common.place_block(block_info, coord, dir, elem_color)
        } else {
            None
        }
    }
}
