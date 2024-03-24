mod game;

use std::{
    collections::HashMap,
    error::Error,
    ffi::{c_char, CStr, CString},
    net::{IpAddr, SocketAddr},
    panic,
    pin::pin,
    ptr::null,
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
};

use futures_util::{StreamExt, TryStreamExt};
use native_dialog::{MessageDialog, MessageType};
use tm_sync_edit_shared::{deserialize, framed_tcp_stream, Map, Message};
use tokio::{
    net::TcpStream,
    runtime, select,
    sync::{oneshot, Notify},
};
use windows_sys::Win32::{
    Foundation::{BOOL, TRUE},
    System::SystemServices::DLL_PROCESS_ATTACH,
};

use crate::game::{
    hook_place_block, hook_place_item, hook_remove_block, hook_remove_item, GameFns,
};

static BLOCK_INFOS: Mutex<Option<HashMap<String, usize>>> = Mutex::new(None);

static ITEM_MODELS: Mutex<Option<HashMap<String, usize>>> = Mutex::new(None);

static JOIN_ERROR: Mutex<Option<CString>> = Mutex::new(None);

static JOIN_STATUS: Mutex<Option<CString>> = Mutex::new(None);

static CANCELLED: Mutex<Option<Arc<Notify>>> = Mutex::new(None);

static OPEN_EDITOR: Mutex<bool> = Mutex::new(false);

static OPEN_EDITOR_RESULT: Mutex<Option<oneshot::Sender<bool>>> = Mutex::new(None);

#[no_mangle]
extern "system" fn DllMain(_dll_module: usize, call_reason: u32, _reserved: usize) -> BOOL {
    if call_reason == DLL_PROCESS_ATTACH {
        // Display a message box when we panic.
        panic::set_hook(Box::new(|panic_info| {
            MessageDialog::new()
                .set_type(MessageType::Error)
                .set_title("SyncEdit.dll")
                .set_text(&panic_info.to_string())
                .show_alert()
                .unwrap();
        }));
    }

    TRUE
}

/// Try to join a server with the given `host` and `port`.
#[no_mangle]
extern "system" fn Join(host: *const c_char, port: *const c_char) {
    *JOIN_ERROR.lock().unwrap() = None;

    // Need to copy string parameters to the heap to safely send them to
    // a new thread since the strings will be dropped when this function returns.
    let host = unsafe { CStr::from_ptr(host).to_owned() };
    let port = unsafe { CStr::from_ptr(port).to_owned() };

    // Initialize the cancellation notification.
    let cancelled = Arc::new(Notify::new());
    *CANCELLED.lock().unwrap() = Some(Arc::clone(&cancelled));

    // Try to join a server asynchronously using a new thread.
    thread::spawn(|| {
        if let Err(error) = join_inner(cancelled, host, port) {
            *JOIN_ERROR.lock().unwrap() = Some(CString::new(error.to_string()).unwrap());
        }
    });
}

/// Cancel joining a server.
#[no_mangle]
extern "system" fn CancelJoin() {
    // Notify the cancellation notification.
    CANCELLED.lock().unwrap().as_ref().unwrap().notify_one();
}

#[no_mangle]
extern "system" fn JoinError() -> *const c_char {
    match &*JOIN_ERROR.lock().unwrap() {
        None => null(),
        // TODO: The returned pointer might be dropped at some point.
        Some(error) => error.as_ptr(),
    }
}

#[no_mangle]
extern "system" fn OpenMapEditor() -> bool {
    let mut open_editor = OPEN_EDITOR.lock().unwrap();

    if *open_editor {
        *open_editor = false;

        true
    } else {
        false
    }
}

#[no_mangle]
extern "system" fn OpenMapEditorResult(success: bool) {
    OPEN_EDITOR_RESULT
        .lock()
        .unwrap()
        .take()
        .unwrap()
        .send(success)
        .unwrap();
}

#[no_mangle]
extern "system" fn RegisterBlockInfo(id: *const c_char, block_info: usize) {
    let id = unsafe { CStr::from_ptr(id).to_str().unwrap().to_owned() };

    let mut block_infos = BLOCK_INFOS.lock().unwrap();

    if block_infos.is_none() {
        *block_infos = Some(HashMap::new());
    }

    block_infos.as_mut().unwrap().insert(id, block_info);
}

#[no_mangle]
extern "system" fn RegisterItemModel(id: *const c_char, item_model: usize) {
    let id = unsafe { CStr::from_ptr(id).to_str().unwrap().to_owned() };

    let mut item_models = ITEM_MODELS.lock().unwrap();

    if item_models.is_none() {
        *item_models = Some(HashMap::new());
    }

    item_models.as_mut().unwrap().insert(id, item_model);
}

#[no_mangle]
extern "system" fn JoinStatus() -> *const c_char {
    match &*JOIN_STATUS.lock().unwrap() {
        None => null(),
        // TODO: The returned pointer might be dropped at some point.
        Some(error) => error.as_ptr(),
    }
}

fn join_inner(cancelled: Arc<Notify>, host: CString, port: CString) -> Result<(), Box<dyn Error>> {
    let host = host.to_str()?;
    let port = u16::from_str(port.to_str()?)?;

    let ip_addr = IpAddr::from_str(host)?;
    let socket_addr = SocketAddr::new(ip_addr, port);

    let runtime = runtime::Builder::new_current_thread().enable_io().build()?;

    runtime.block_on(async {
        // We either complete the join task or cancel it.
        select! {
            result = join_inner_inner(socket_addr) => result,
            _ = cancelled.notified() => Ok(()),
        }
    })
}

async fn join_inner_inner(socket_addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    *JOIN_STATUS.lock().unwrap() = Some(CString::new("Connecting...").unwrap());

    let tcp_stream = TcpStream::connect(socket_addr).await?;

    let mut framed_tcp_stream = pin!(framed_tcp_stream(tcp_stream).peekable());

    let frame = framed_tcp_stream.try_next().await?.ok_or("")?.freeze();
    let map: Map = tm_sync_edit_shared::deserialize(&frame)?;

    select! {
        None = framed_tcp_stream.as_mut().peek() => return Err("".into()),
        result = join_inner_inner_inner() => result?
    }

    let game_fns = GameFns::find()?;

    *JOIN_STATUS.lock().unwrap() = Some(CString::new("Connected").unwrap());

    let _place_block_hook = hook_place_block()?;
    let _remove_block_hook = hook_remove_block()?;
    let _place_item_hook = hook_place_item()?;
    let _remove_item_hook = hook_remove_item()?;

    loop {
        select! {
            result = framed_tcp_stream.try_next() => match result? {
                None => return Err("".into()),
                Some(frame) => {
                    let frame = frame.freeze();

                    let message: Message = deserialize(&frame)?;

                    match message {
                        Message::PlaceBlock => {}
                        Message::RemoveBlock => {}
                        Message::PlaceItem => {}
                        Message::RemoveItem => {}
                        Message::AddCustomBlockInfo => {}
                        Message::AddCustomItemModel => {}
                    }
                }
            }
        }
    }
}

async fn join_inner_inner_inner() -> Result<(), Box<dyn Error>> {
    let (sender, receiver) = oneshot::channel();
    *OPEN_EDITOR_RESULT.lock().unwrap() = Some(sender);
    *OPEN_EDITOR.lock().unwrap() = true;

    let open_editor_result = receiver.await?;

    if !open_editor_result {
        return Err("".into());
    }

    Ok(())
}
