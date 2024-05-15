mod game;

mod os {
    mod executable_memory;
    mod process;

    pub use executable_memory::ExecutableMemory;
    pub use process::Process;
}

use std::{
    error::Error,
    ffi::{c_char, c_void, CStr},
    fs,
    future::{poll_fn, Future},
    net::{IpAddr, SocketAddr},
    num::NonZeroUsize,
    panic::{self},
    path::PathBuf,
    pin::Pin,
    str::FromStr,
    task::{self, Poll},
};

use ahash::AHashMap;
use async_compat::CompatExt;
use futures::{executor::block_on, task::noop_waker_ref, SinkExt, TryStreamExt};
use game::{
    cast_nod, fids_folder_full_path, fids_folder_get_subfolder, hook_place_block, hook_place_item,
    hook_remove_block, hook_remove_item, Block, BlockInfo, FidFile, FidsFolder, IdNameFn, Item,
    ItemModel, ManiaPlanet, MapEditor, Nod, NodRef, PlaceBlockFn, PlaceItemFn, PlaceNormalBlockFn,
    PreloadBlockInfoFn, PreloadFidFn, RemoveBlockFn, RemoveItemFn,
};
use gamebox::{
    engines::game::map::{Direction, ElemColor},
    Vec3,
};
use native_dialog::{MessageDialog, MessageType};
use os::Process;
use shared::{
    deserialize, framed_tcp_stream, serialize, BlockDesc, BlockDescKind, FramedTcpStream, ItemDesc,
    MapDesc, Message, ModelId,
};
use tokio::{net::TcpStream, select};
use windows_sys::Win32::{
    Foundation::{BOOL, HINSTANCE, TRUE},
    System::SystemServices::DLL_PROCESS_ATTACH,
};

/// DLL entry point.
#[no_mangle]
unsafe extern "system" fn DllMain(
    _module: HINSTANCE,
    call_reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    if call_reason == DLL_PROCESS_ATTACH {
        // Display an error message box when panicking.
        panic::set_hook(Box::new(|panic_info| {
            let _ = MessageDialog::new()
                .set_type(MessageType::Error)
                .set_title("SyncEdit.dll")
                .set_text(&panic_info.to_string())
                .show_alert();
        }));
    }

    TRUE
}

#[no_mangle]
unsafe extern "system" fn CreateContext() -> *mut Context {
    let mut context = Context::new();
    context.set_status_text("Disconnected");

    Box::into_raw(Box::new(context))
}

#[no_mangle]
unsafe extern "system" fn DestroyContext(context: *mut Context) {
    drop(Box::from_raw(context));
}

#[no_mangle]
unsafe extern "system" fn OpenConnection(
    context: *mut Context,
    host: *const c_char,
    port: *const c_char,
    mania_planet: *mut ManiaPlanet,
    game_folder: *mut FidsFolder,
) {
    (*context).state = State::Connecting;

    let host = str_from_c_str(host).to_owned();
    let port = str_from_c_str(port).to_owned();

    let connection_future = Box::pin(connection(
        &mut *context,
        host,
        port,
        &mut *mania_planet,
        &mut *game_folder,
    ));

    (*context).connection_future = Some(connection_future);
}

#[no_mangle]
unsafe extern "system" fn UpdateConnection(context: &mut Context) {
    let connection_future = context
        .connection_future
        .as_mut()
        .expect("no open connection");

    let mut task_context = task::Context::from_waker(noop_waker_ref());

    if let Poll::Ready(result) = connection_future.as_mut().poll(&mut task_context) {
        context.state = State::Disconnected;

        if let Err(err) = result {
            context.set_status_text(&err.to_string());
        }
    }
}

#[no_mangle]
unsafe extern "system" fn CloseConnection(context: &mut Context) {
    context.state = State::Disconnected;
    context.map_editor = None;
    context.connection_future = None;
}

#[repr(C)]
struct Context {
    state: State,
    status_text_buf: Box<[u8; 256]>,
    map_editor: Option<NonZeroUsize>,
    should_open_editor: bool,

    connection_future: Option<ConnectionFuture>,
    framed_tcp_stream: Option<FramedTcpStream>,
    game_block_infos: Option<AHashMap<String, NodRef<BlockInfo>>>,
    game_item_models: Option<AHashMap<String, NodRef<ItemModel>>>,
    id_name_fn: Option<IdNameFn>,
    place_block_fn: Option<PlaceBlockFn>,
    place_normal_block_fn: Option<PlaceNormalBlockFn>,
    remove_block_fn: Option<RemoveBlockFn>,
    place_item_fn: Option<PlaceItemFn>,
    remove_item_fn: Option<RemoveItemFn>,
    hooks_enabled: bool,
    blocks: Option<AHashMap<BlockDesc, NodRef<Block>>>,
    items: Option<AHashMap<ItemDesc, NodRef<Item>>>,
}

impl Context {
    fn new() -> Self {
        Self {
            state: State::Disconnected,
            status_text_buf: Box::new([0; 256]),
            map_editor: None,
            should_open_editor: false,

            connection_future: None,
            framed_tcp_stream: None,
            game_block_infos: None,
            game_item_models: None,
            id_name_fn: None,
            place_block_fn: None,
            place_normal_block_fn: None,
            remove_block_fn: None,
            place_item_fn: None,
            remove_item_fn: None,
            hooks_enabled: true,
            blocks: None,
            items: None,
        }
    }

    fn set_status_text(&mut self, status_text: &str) {
        if status_text.len() >= self.status_text_buf.len() {
            panic!("status text is too long for buffer");
        }

        self.status_text_buf[..status_text.len()].copy_from_slice(status_text.as_bytes());
        self.status_text_buf[status_text.len()] = 0;
    }
}

#[repr(u8)]
enum State {
    Disconnected,
    Connecting,
    Connected,
}

type ConnectionFuture = Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>>>>;

async fn connection(
    context: &mut Context,
    host: String,
    port: String,
    mania_planet: &mut ManiaPlanet,
    game_folder: &mut FidsFolder,
) -> Result<(), Box<dyn Error>> {
    let ip_addr = IpAddr::from_str(&host)?;
    let port = u16::from_str(&port)?;
    let socket_addr = SocketAddr::new(ip_addr, port);

    context.set_status_text("Connecting...");

    let tcp_stream = TcpStream::connect(socket_addr).compat().await?;

    context.framed_tcp_stream = Some(framed_tcp_stream(tcp_stream));

    let frame = context
        .framed_tcp_stream
        .as_mut()
        .unwrap()
        .try_next()
        .await?
        .ok_or("server closed connection")?;

    let map_desc: MapDesc = deserialize(&frame)?;

    context.set_status_text("Opening map editor...");

    open_map_editor(context).await;

    let process = Process::open_current()?;
    let exe_module_memory = process.main_module_memory()?;

    let preload_fid_fn = PreloadFidFn::find(exe_module_memory)?;
    let preload_block_info_fn = PreloadBlockInfoFn::find(exe_module_memory)?;

    let mut game_block_infos = AHashMap::new();
    let mut game_item_models = AHashMap::new();

    load_game_models(
        game_folder,
        &mut game_block_infos,
        &mut game_item_models,
        exe_module_memory,
        preload_fid_fn,
    )?;

    context.game_block_infos = Some(game_block_infos);
    context.game_item_models = Some(game_item_models);

    load_custom_block_models(
        game_folder,
        map_desc.custom_block_models,
        preload_fid_fn,
        preload_block_info_fn,
    )?;

    load_custom_item_models(game_folder, map_desc.custom_item_models, preload_fid_fn)?;

    context.place_block_fn = Some(PlaceBlockFn::find(exe_module_memory)?);
    context.place_normal_block_fn = Some(PlaceNormalBlockFn::find(exe_module_memory)?);
    context.remove_block_fn = Some(RemoveBlockFn::find(exe_module_memory)?);
    context.place_item_fn = Some(PlaceItemFn::find(exe_module_memory)?);
    context.remove_item_fn = Some(RemoveItemFn::find(exe_module_memory)?);

    // for block_desc in map_desc.blocks {
    //     handle_place_block(context, &block_desc)?;
    // }

    // for item_desc in map_desc.items {
    //     handle_place_item(context, &item_desc)?;
    // }

    context.state = State::Connected;
    context.set_status_text("Connected");

    context.blocks = Some(AHashMap::new());
    context.items = Some(AHashMap::new());

    context.id_name_fn = Some(IdNameFn::find(exe_module_memory)?);

    handle_place_block(
        context,
        &BlockDesc {
            model_id: ModelId::Game {
                name: "RoadTechStraight".to_owned(),
            },
            variant_index: 0,
            elem_color: ElemColor::Default,
            kind: BlockDescKind::Normal {
                coordinate: Vec3 {
                    x: 20,
                    y: 20,
                    z: 20,
                },
                direction: Direction::North,
                is_ground: false,
                is_ghost: false,
            },
        },
    )
    .unwrap();

    // let _place_block_hook = hook_place_block(context, place_block_callback)?;
    // let _remove_block_hook = hook_remove_block(context, remove_block_callback)?;
    // let _place_item_hook = hook_place_item(context, place_item_callback)?;
    // let _remove_item_hook = hook_remove_item(context, remove_item_callback)?;

    loop {
        select! {
            result = context.framed_tcp_stream.as_mut().unwrap().try_next() => match result? {
                None => return Err("server closed connection".into()),
                Some(frame) => handle_frame(context, &frame).await?,
            }
        }
    }
}

/// Load all the [BlockInfo]'s and [ItemModel]'s that are internal to the game.
fn load_game_models(
    game_folder: &FidsFolder,
    block_infos: &mut AHashMap<String, NodRef<BlockInfo>>,
    item_models: &mut AHashMap<String, NodRef<ItemModel>>,
    exe_module_memory: &[u8],
    preload_fid_fn: PreloadFidFn,
) -> Result<(), Box<dyn Error>> {
    let id_name_fn = IdNameFn::find(exe_module_memory)?;

    let game_data_folder = fids_folder_get_subfolder(game_folder, "GameData")
        .ok_or("failed to find GameData folder")?;

    let stadium_folder = fids_folder_get_subfolder(game_data_folder, "Stadium")
        .ok_or("failed to find Stadium folder")?;

    let block_info_folder = fids_folder_get_subfolder(stadium_folder, "GameCtnBlockInfo")
        .ok_or("failed to find GameCtnBlockInfo folder")?;

    load_block_infos(block_info_folder, block_infos, preload_fid_fn, id_name_fn)?;

    let items_folder =
        fids_folder_get_subfolder(stadium_folder, "Items").ok_or("failed to find Items folder")?;

    load_item_models(items_folder, item_models, preload_fid_fn, id_name_fn)?;

    Ok(())
}

/// Recursively load all the [BlockInfo]'s in the given `folder`.
fn load_block_infos(
    folder: &FidsFolder,
    block_infos: &mut AHashMap<String, NodRef<BlockInfo>>,
    preload_fid_fn: PreloadFidFn,
    id_name_fn: IdNameFn,
) -> Result<(), Box<dyn Error>> {
    for subfolder in folder.trees() {
        load_block_infos(subfolder, block_infos, preload_fid_fn, id_name_fn)?;
    }

    for file in folder.leaves() {
        let nod = unsafe {
            preload_fid_fn
                .call(*file as *const _ as _)
                .ok_or("failed to preload fid")?
        };

        if let Some(block_info) = cast_nod::<BlockInfo>(nod) {
            let id_name = id_name_fn.call(block_info.id);

            block_infos.insert(id_name, NodRef::from(block_info));
        }
    }

    Ok(())
}

/// Recursively load all the [ItemModel]'s in the given `folder`.
fn load_item_models(
    folder: &FidsFolder,
    item_models: &mut AHashMap<String, NodRef<ItemModel>>,
    preload_fid_fn: PreloadFidFn,
    id_name_fn: IdNameFn,
) -> Result<(), Box<dyn Error>> {
    for subfolder in folder.trees() {
        load_item_models(subfolder, item_models, preload_fid_fn, id_name_fn)?;
    }

    for file in folder.leaves() {
        let nod = unsafe {
            preload_fid_fn
                .call(*file as *const _ as _)
                .ok_or("failed to preload fid")?
        };

        if let Some(item_model) = cast_nod::<ItemModel>(nod) {
            let id_name = id_name_fn.call(item_model.id);

            item_models.insert(id_name, NodRef::from(item_model));
        }
    }

    Ok(())
}

fn load_custom_block_models(
    folder: &mut FidsFolder,
    item_models_gbx: Vec<Vec<u8>>,
    preload_fid_fn: PreloadFidFn,
    preload_block_info_fn: PreloadBlockInfoFn,
) -> Result<(), Box<dyn Error>> {
    let mut file_paths = vec![];

    let folder_path: PathBuf = fids_folder_full_path(folder);

    for item_model_gbx in item_models_gbx {
        let hash = blake3::hash(&item_model_gbx);

        let mut file_path = folder_path.clone();
        file_path.push(hash.to_string());
        file_path.set_extension("Block.Gbx");

        fs::write(&file_path, item_model_gbx)?;

        file_paths.push(file_path);
    }

    folder.update_tree(false);

    for file_path in file_paths {
        let file_name = file_path.file_name().unwrap();

        let file = folder
            .leaves()
            .iter()
            .find(|file| file.name() == file_name)
            .copied()
            .unwrap();

        let item_model = unsafe {
            &mut *(preload_fid_fn
                .call(file as *const FidFile as *mut FidFile)
                .ok_or("failed to preload fid")? as *mut Nod as *mut ItemModel)
        };

        preload_block_info_fn.call(item_model, file);

        let block_info = item_model.entity_model().unwrap();

        fs::remove_file(file_path)?;
    }

    Ok(())
}

fn load_custom_item_models(
    folder: &mut FidsFolder,
    item_models_gbx: Vec<Vec<u8>>,
    preload_fid_fn: PreloadFidFn,
) -> Result<(), Box<dyn Error>> {
    let mut file_paths = vec![];

    let folder_path: PathBuf = fids_folder_full_path(folder);

    for item_model_gbx in item_models_gbx {
        let hash = blake3::hash(&item_model_gbx);

        let mut file_path = folder_path.clone();
        file_path.push(hash.to_string());
        file_path.set_extension("Item.Gbx");

        fs::write(&file_path, item_model_gbx)?;

        file_paths.push(file_path);
    }

    folder.update_tree(false);

    for file_path in file_paths {
        let file_name = file_path.file_name().unwrap();

        let file = folder
            .leaves()
            .iter()
            .find(|file| file.name() == file_name)
            .copied()
            .unwrap();

        let item_model = unsafe {
            preload_fid_fn
                .call(file as *const FidFile as *mut FidFile)
                .ok_or("failed to preload fid")?
        };

        fs::remove_file(file_path)?;
    }

    Ok(())
}

async fn open_map_editor(context: &mut Context) {
    context.map_editor = None;
    context.should_open_editor = true;

    let future = poll_fn(|_| {
        if context.map_editor.is_some() {
            context.should_open_editor = false;

            Poll::Ready(())
        } else {
            Poll::Pending
        }
    });

    future.await;
}

fn handle_place_block(context: &mut Context, block_desc: &BlockDesc) -> Result<(), Box<dyn Error>> {
    let map_editor = unsafe { &mut *(context.map_editor.unwrap().get() as *mut MapEditor) };

    let block_info = match block_desc.model_id {
        ModelId::Game { ref name } => context
            .game_block_infos
            .as_mut()
            .unwrap()
            .get(name)
            .ok_or("failed to find block info with the given name")?,
        ModelId::Custom { .. } => {
            todo!()
        }
    };

    match block_desc.kind {
        BlockDescKind::Normal {
            coordinate,
            direction,
            is_ground,
            is_ghost: false,
        } => {
            unsafe {
                context.place_normal_block_fn.as_mut().unwrap().call(
                    map_editor,
                    block_info,
                    coordinate,
                    direction,
                    block_desc.elem_color,
                );
            };
        }
        _ => {}
    }

    Ok(())
}

fn handle_place_item(context: &mut Context, item_desc: &ItemDesc) -> Result<(), Box<dyn Error>> {
    let map_editor = unsafe { &mut *(context.map_editor.unwrap().get() as *mut MapEditor) };

    let item_model = match item_desc.model_id {
        ModelId::Game { ref name } => context
            .game_item_models
            .as_mut()
            .unwrap()
            .get(name)
            .ok_or("failed to find item model with the given name")?,
        ModelId::Custom { .. } => {
            todo!()
        }
    };

    unsafe {
        context.place_item_fn.as_mut().unwrap().call(
            map_editor,
            item_model,
            item_desc.yaw,
            item_desc.pitch,
            item_desc.roll,
            item_desc.position.clone(),
            item_desc.pivot_position.clone(),
            item_desc.elem_color,
            item_desc.anim_offset,
        )
    };

    Ok(())
}

unsafe extern "system" fn place_block_callback(context: &mut Context, block: Option<&Block>) {}

unsafe extern "system" fn remove_block_callback(context: &mut Context, block: &Block) {
    block_on(async {
        if !context.hooks_enabled {
            return;
        }

        let kind = if block.is_free() {
            BlockDescKind::Free {
                position: block.position.clone(),
                yaw: block.yaw,
                pitch: block.pitch,
                roll: block.roll,
            }
        } else {
            BlockDescKind::Normal {
                coordinate: Vec3 {
                    x: block.coordinate.x as u8,
                    y: block.coordinate.y as u8,
                    z: block.coordinate.z as u8,
                },
                direction: block.direction,
                is_ground: block.is_ground(),
                is_ghost: block.is_ghost(),
            }
        };

        let name = context.id_name_fn.unwrap().call(block.block_info().id);

        let block_desc = BlockDesc {
            model_id: ModelId::Game { name },
            variant_index: block.variant_index(),
            elem_color: block.elem_color,
            kind,
        };

        context.blocks.as_mut().unwrap().remove(&block_desc);

        let message = Message::RemoveBlock(block_desc);

        let frame = serialize(&message).unwrap();

        context
            .framed_tcp_stream
            .as_mut()
            .unwrap()
            .send(frame.into())
            .await
            .unwrap();
    })
}

unsafe extern "system" fn place_item_callback(
    context: &mut Context,
    item: *const Item,
    success: bool,
) {
    block_on(async {
        if !context.hooks_enabled {
            return;
        }

        if success {
            let item = &*item;

            let name = context.id_name_fn.unwrap().call(item.model().id);

            let params = &item.params;

            let item_desc = ItemDesc {
                model_id: ModelId::Game { name },
                position: params.position.clone(),
                yaw: params.yaw,
                pitch: params.pitch,
                roll: params.roll,
                pivot_position: params.pivot_position.clone(),
                elem_color: params.elem_color,
                anim_offset: params.anim_offset,
            };

            context
                .items
                .as_mut()
                .unwrap()
                .insert(item_desc.clone(), NodRef::from(item));

            let message = Message::PlaceItem(item_desc);

            let frame = serialize(&message).unwrap();

            context
                .framed_tcp_stream
                .as_mut()
                .unwrap()
                .send(frame.into())
                .await
                .unwrap();
        }
    })
}

unsafe extern "system" fn remove_item_callback(context: &mut Context, item: &Item) {
    block_on(async {
        if !context.hooks_enabled {
            return;
        }

        let name = context.id_name_fn.unwrap().call(item.model().id);

        let params = &item.params;

        let item_desc = ItemDesc {
            model_id: ModelId::Game { name },
            position: params.position.clone(),
            yaw: params.yaw,
            pitch: params.pitch,
            roll: params.roll,
            pivot_position: params.pivot_position.clone(),
            elem_color: params.elem_color,
            anim_offset: params.anim_offset,
        };

        context.items.as_mut().unwrap().remove(&item_desc);

        let message = Message::RemoveItem(item_desc);

        let frame = serialize(&message).unwrap();

        context
            .framed_tcp_stream
            .as_mut()
            .unwrap()
            .send(frame.into())
            .await
            .unwrap();
    })
}

async fn handle_frame(context: &mut Context, frame: &[u8]) -> Result<(), Box<dyn Error>> {
    let message = deserialize(frame)?;

    let map_editor = unsafe { &mut *(context.map_editor.unwrap().get() as *mut MapEditor) };

    context.hooks_enabled = false;

    match message {
        Message::PlaceBlock(block_desc) => handle_place_block(context, &block_desc)?,
        Message::RemoveBlock(block_desc) => {
            if let Some(block) = context.blocks.as_mut().unwrap().get_mut(&block_desc) {
                unsafe {
                    context
                        .remove_block_fn
                        .as_mut()
                        .unwrap()
                        .call(map_editor, &mut *block)
                };
            }
        }
        Message::PlaceItem(item_desc) => handle_place_item(context, &item_desc)?,
        Message::RemoveItem(item_desc) => {
            if let Some(item) = context.items.as_mut().unwrap().get_mut(&item_desc) {
                unsafe {
                    context
                        .remove_item_fn
                        .as_mut()
                        .unwrap()
                        .call(map_editor, &mut *item)
                };
            }
        }
        Message::AddBlockModel { .. } => {}
        Message::AddItemModel { .. } => {}
    }

    context.hooks_enabled = true;

    Ok(())
}

unsafe fn str_from_c_str<'a>(c_string: *const c_char) -> &'a str {
    CStr::from_ptr(c_string)
        .to_str()
        .expect("invalid UTF-8 string")
}
