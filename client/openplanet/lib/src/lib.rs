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
    future::{poll_fn, Future},
    net::{IpAddr, SocketAddr},
    num::NonZeroUsize,
    panic,
    pin::Pin,
    str::FromStr,
    task::{self, Poll},
};

use ahash::AHashMap;
use async_compat::CompatExt;
use futures::{task::noop_waker_ref, TryStreamExt};
use game::{
    cast_nod, BlockInfo, FidsFolder, IdNameFn, ItemModel, MapEditor, NodRef, PlaceBlockFn,
    PlaceItemFn, PreloadFidFn,
};
use native_dialog::{MessageDialog, MessageType};
use os::Process;
use shared::{deserialize, framed_tcp_stream, BlockDescKind, MapDesc, Message, ModelId};
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
    game_folder: *const FidsFolder,
) {
    (*context).state = State::Connecting;

    let host = str_from_c_str(host).to_owned();
    let port = str_from_c_str(port).to_owned();

    let connection_future = Box::pin(connection(&mut *context, host, port, &*game_folder));

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
}

impl Context {
    fn new() -> Self {
        Self {
            state: State::Disconnected,
            status_text_buf: Box::new([0; 256]),
            map_editor: None,
            should_open_editor: false,

            connection_future: None,
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
    game_folder: &FidsFolder,
) -> Result<(), Box<dyn Error>> {
    let ip_addr = IpAddr::from_str(&host)?;
    let port = u16::from_str(&port)?;
    let socket_addr = SocketAddr::new(ip_addr, port);

    context.set_status_text("Connecting...");

    let tcp_stream = TcpStream::connect(socket_addr).compat().await?;

    let mut framed_tcp_stream = framed_tcp_stream(tcp_stream);

    let frame = framed_tcp_stream
        .try_next()
        .await?
        .ok_or("server closed connection")?;

    let map_desc: MapDesc = deserialize(&frame)?;

    context.set_status_text("Opening map editor...");

    open_map_editor(context).await;

    let process = Process::open_current()?;
    let exe_module_memory = process.main_module_memory()?;

    let mut game_block_infos = AHashMap::new();
    let mut game_item_models = AHashMap::new();

    load_game_models(
        game_folder,
        &mut game_block_infos,
        &mut game_item_models,
        exe_module_memory,
    )?;

    let place_block_fn = PlaceBlockFn::find(exe_module_memory)?;
    let place_item_fn = PlaceItemFn::find(exe_module_memory)?;

    let map_editor = unsafe { &mut *(context.map_editor.unwrap().get() as *mut MapEditor) };

    for block_desc in map_desc.blocks {
        let block_info = match block_desc.model_id {
            ModelId::Game { ref name } => game_block_infos
                .get(name)
                .ok_or("failed to find block info with the given name")?,
            ModelId::Custom { .. } => {
                todo!()
            }
        };

        match block_desc.kind {
            BlockDescKind::Normal {
                x,
                y,
                z,
                direction,
                is_ground,
                is_ghost,
            } => {
                unsafe {
                    place_block_fn.call_normal(
                        map_editor,
                        block_info,
                        x,
                        y,
                        z,
                        direction,
                        block_desc.elem_color,
                        is_ghost,
                        is_ground,
                    )
                };
            }
            BlockDescKind::Free {
                x,
                y,
                z,
                yaw,
                pitch,
                roll,
            } => {
                unsafe {
                    place_block_fn.call_free(
                        map_editor,
                        block_info,
                        block_desc.elem_color,
                        x,
                        y,
                        z,
                        yaw,
                        pitch,
                        roll,
                    )
                };
            }
        }
    }

    for item_desc in map_desc.items {
        let item_model = match item_desc.model_id {
            ModelId::Game { ref name } => game_item_models
                .get(name)
                .ok_or("failed to find item model with the given name")?,
            ModelId::Custom { .. } => {
                todo!()
            }
        };

        unsafe {
            place_item_fn.call(
                map_editor,
                item_model,
                item_desc.yaw,
                item_desc.pitch,
                item_desc.roll,
                item_desc.x,
                item_desc.y,
                item_desc.z,
            )
        };
    }

    context.state = State::Connected;
    context.set_status_text("Connected");

    loop {
        select! {
            result = framed_tcp_stream.try_next() => match result? {
                None => return Err("server closed connection".into()),
                Some(frame) => handle_frame( &frame).await?,
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
) -> Result<(), Box<dyn Error>> {
    let preload_fid_fn = PreloadFidFn::find(exe_module_memory)?;
    let id_name_fn = IdNameFn::find(exe_module_memory)?;

    let game_data_folder = game_folder
        .trees()
        .iter()
        .find(|folder| folder.name() == "GameData")
        .ok_or("failed to find GameData folder")?;

    let stadium_folder = game_data_folder
        .trees()
        .iter()
        .find(|folder| folder.name() == "Stadium")
        .ok_or("failed to find Stadium folder")?;

    let block_info_folder = stadium_folder
        .trees()
        .iter()
        .find(|folder| folder.name() == "GameCtnBlockInfo")
        .ok_or("failed to find GameCtnBlockInfo folder")?;

    load_block_infos(block_info_folder, block_infos, preload_fid_fn, id_name_fn)?;

    let items_folder = stadium_folder
        .trees()
        .iter()
        .find(|folder| folder.name() == "Items")
        .ok_or("failed to find Items folder")?;

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

async fn open_map_editor(context: &mut Context) {
    context.map_editor = None;
    context.should_open_editor = true;

    let future = poll_fn(|_cx| {
        if context.map_editor.is_some() {
            context.should_open_editor = false;

            Poll::Ready(())
        } else {
            Poll::Pending
        }
    });

    future.await;
}

async fn handle_frame(frame: &[u8]) -> Result<(), Box<dyn Error>> {
    let message = deserialize(frame)?;

    match message {
        Message::PlaceBlock(..) => {}
        Message::RemoveBlock(..) => {}
        Message::PlaceItem(..) => {}
        Message::RemoveItem(..) => {}
        Message::AddBlockModel { .. } => {}
        Message::AddItemModel { .. } => {}
    }

    Ok(())
}

unsafe fn str_from_c_str<'a>(c_string: *const c_char) -> &'a str {
    CStr::from_ptr(c_string)
        .to_str()
        .expect("invalid UTF-8 string")
}
