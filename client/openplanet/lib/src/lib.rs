mod game;
mod os;

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
use futures::{executor::block_on, task::noop_waker_ref, SinkExt, TryStreamExt};
use game::{
    hook_place_block, hook_place_item, hook_remove_block, hook_remove_item, Block, BlockInfo,
    FidsFolder, GameFns, IdNameFn, Item, ItemModel, ItemParams,
};
use native_dialog::{MessageDialog, MessageType};
use shared::{
    deserialize, framed_tcp_stream, serialize, BlockDesc, BlockDescKind, FramedTcpStream, ItemDesc,
    Message, ModelId,
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

/// Create a new context.
#[no_mangle]
unsafe extern "system" fn CreateContext() -> *mut Context {
    let mut context = Context::new();
    context.set_status_text("Disconnected");

    Box::into_raw(Box::new(context))
}

/// Destroy the given `context`.
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
unsafe extern "system" fn CloseConnection(context: *mut Context) {
    let context = &mut *context;

    context.state = State::Disconnected;
    context.connection_future = None;
    context.framed_tcp_stream = None;
}

#[repr(C)]
struct Context {
    state: State,
    status_text_buf: Box<[u8; 256]>,
    map_editor: Option<NonZeroUsize>,

    connection_future: Option<ConnectionFuture>,
    framed_tcp_stream: Option<FramedTcpStream>,
}

impl Context {
    fn new() -> Self {
        Self {
            state: State::Disconnected,
            status_text_buf: Box::new([0; 256]),
            map_editor: None,

            connection_future: None,
            framed_tcp_stream: None,
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
    OpeningMapEditor,
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

    context.set_status_text("Opening map editor...");

    open_map_editor(context).await;

    context.state = State::Connected;
    context.set_status_text("Connected");

    context.framed_tcp_stream = Some(framed_tcp_stream(tcp_stream));

    let _place_block_hook = hook_place_block(context, place_block_callback)?;
    let _remove_block_hook = hook_remove_block(context, remove_block_callback)?;
    let _place_item_hook = hook_place_item(context, place_item_callback)?;
    let _remove_item_hook = hook_remove_item(context, remove_item_callback)?;

    loop {
        select! {
            result = context.framed_tcp_stream.as_mut().unwrap().try_next() => match result? {
                None => return Err("server closed connection".into()),
                Some(frame) => handle_frame(context, &frame).await?,
            }
        }
    }
}

async fn open_map_editor(context: &mut Context) {
    context.map_editor = None;
    context.state = State::OpeningMapEditor;

    let future = poll_fn(|_cx| {
        if context.map_editor.is_some() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    });

    future.await;
}

async fn handle_frame(context: &mut Context, frame: &[u8]) -> Result<(), Box<dyn Error>> {
    let message = deserialize(frame)?;

    match message {
        Message::PlaceBlock(block_desc) => {}
        Message::RemoveBlock(block_desc) => {}
        Message::PlaceItem(item_desc) => {}
        Message::RemoveItem(item_desc) => {}
        Message::AddBlockModel { .. } => {}
        Message::AddItemModel { .. } => {}
    }

    Ok(())
}

unsafe extern "system" fn place_block_callback(context: &mut Context, block: Option<&Block>) {}

unsafe extern "system" fn remove_block_callback(context: &mut Context, block: *mut Block) {}

unsafe extern "system" fn place_item_callback(
    context: &mut Context,
    item_model: *mut ItemModel,
    item_params: *mut ItemParams,
) {
}

unsafe extern "system" fn remove_item_callback(context: &mut Context, item: *mut Item) {}

unsafe fn str_from_c_str<'a>(c_string: *const c_char) -> &'a str {
    CStr::from_ptr(c_string)
        .to_str()
        .expect("invalid UTF-8 string")
}

fn get_fids_subfolder<'a>(folder: &'a FidsFolder, name: &str) -> Option<&'a FidsFolder> {
    folder
        .trees()
        .iter()
        .find(|subfolder| subfolder.name() == name)
        .copied()
}

fn load_game_objects(
    context: &mut Context,
    game_folder: &FidsFolder,
) -> Result<(), Box<dyn Error>> {
    let game_data_folder =
        get_fids_subfolder(game_folder, "GameData").ok_or("could not find folder GameData")?;

    let stadium_folder = get_fids_subfolder(game_data_folder, "Stadium")
        .ok_or("could not find folder GameData/Stadium")?;

    let block_infos_folder = get_fids_subfolder(stadium_folder, "GameCtnBlockInfo")
        .ok_or("could not find folder GameData/Stadium/CGameCtnBlockInfo")?;

    let items_folder = get_fids_subfolder(stadium_folder, "Items")
        .ok_or("could to find folder GameData/Stadium/Items")?;

    load_game_block_infos(context, block_infos_folder);
    load_game_item_models(context, items_folder);

    Ok(())
}

fn load_game_block_infos(context: &mut Context, folder: &FidsFolder) {
    for fid in folder.leaves() {
        if !fid.nod.is_null() {
            let class_id = unsafe { (*fid.nod).class_id() };

            if class_id == 0x0304f000
                || class_id == 0x03051000
                || class_id == 0x03053000
                || class_id == 0x03340000
                || class_id == 0x0335B000
            {
                let block_info = unsafe { &mut *(fid.nod as *mut BlockInfo) };

                // let block_info_id_name = context.id_name_fn.as_ref().unwrap().call(block_info.id);

                // context
                //     .block_infos
                //     .insert(block_info_id_name.to_owned(), block_info);
            }
        }
    }

    for subfolder in folder.trees() {
        load_game_block_infos(context, subfolder);
    }
}

fn load_game_item_models(context: &mut Context, folder: &FidsFolder) {
    for fid in folder.leaves() {
        if !fid.nod.is_null() {
            let class_id = unsafe { (*fid.nod).class_id() };

            if class_id == 0x2e002000 {
                let item_model = unsafe { &mut *(fid.nod as *mut ItemModel) };

                // let item_model_id_name = context.id_name_fn.as_ref().unwrap().call(item_model.id);

                // context
                //     .item_models
                //     .insert(item_model_id_name.to_owned(), item_model);
            }
        }
    }

    for subfolder in folder.trees() {
        load_game_item_models(context, subfolder);
    }
}

fn block_desc_from_block(context: &Context, block: &Block) -> BlockDesc {
    let block_info = block.block_info();

    // let block_info_id_name = context.id_name_fn.as_ref().unwrap().call(block_info.id);

    let model_id = ModelId::Game {
        name: "".to_owned(),
    };

    let kind = if !block.flags.is_free() {
        BlockDescKind::Normal {
            x: block.x_coord as u8,
            y: block.y_coord as u8,
            z: block.z_coord as u8,
            direction: block.direction,
            is_ground: block.flags.is_ground(),
            is_ghost: block.flags.is_ghost(),
        }
    } else {
        BlockDescKind::Free {
            x: block.x_pos,
            y: block.y_pos,
            z: block.z_pos,
            yaw: block.yaw,
            pitch: block.pitch,
            roll: block.roll,
        }
    };

    BlockDesc {
        model_id,
        elem_color: block.elem_color,
        kind,
    }
}

fn item_desc_from_item(context: &Context, model: &ItemModel, params: &ItemParams) -> ItemDesc {
    // let item_model_id_name = context.id_name_fn.as_ref().unwrap().call(model.id);

    let model_id = ModelId::Game {
        name: "".to_owned(),
    };

    ItemDesc {
        model_id,
        x: params.x_pos,
        y: params.y_pos,
        z: params.z_pos,
        yaw: params.yaw,
        pitch: params.pitch,
        roll: params.roll,
    }
}
