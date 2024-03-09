use std::{
    collections::HashMap,
    error::Error,
    ffi::{c_char, CStr, CString},
    io,
    mem::{size_of, transmute, MaybeUninit},
    net::{IpAddr, SocketAddr},
    panic,
    pin::pin,
    ptr::null,
    slice,
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
};

use futures_util::{StreamExt, TryStreamExt};
use memchr::memmem;
use native_dialog::{MessageDialog, MessageType};
use tm_sync_edit_shared::{framed_tcp_stream, Map};
use tokio::{
    net::TcpStream,
    runtime, select,
    sync::{oneshot, Notify},
};
use windows_sys::Win32::{
    Foundation::{BOOL, TRUE},
    System::{
        LibraryLoader::GetModuleHandleW,
        ProcessStatus::{GetModuleInformation, MODULEINFO},
        SystemServices::DLL_PROCESS_ATTACH,
        Threading::GetCurrentProcess,
    },
};

static BLOCK_INFOS: Mutex<Option<HashMap<String, usize>>> = Mutex::new(None);

static ITEM_MODELS: Mutex<Option<HashMap<String, usize>>> = Mutex::new(None);

static JOIN_ERROR: Mutex<Option<CString>> = Mutex::new(None);

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
    let tcp_stream = TcpStream::connect(socket_addr).await?;

    let mut framed_tcp_stream = pin!(framed_tcp_stream(tcp_stream).peekable());

    let frame = framed_tcp_stream.try_next().await?.ok_or("")?.freeze();
    let map: Map = tm_sync_edit_shared::deserialize(&frame)?;

    select! {
        None = framed_tcp_stream.as_mut().peek() => return Err("".into()),
        result = join_inner_inner_inner() => result?
    }

    let game_fns = GameFns::find()?;

    loop {
        select! {
            result = framed_tcp_stream.try_next() => match result? {
                None => return Err("".into()),
                Some(frame) => {
                    let frame = frame.freeze();
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

struct GameFns {
    place_block_fn: unsafe extern "system" fn(),
    remove_block_fn: unsafe extern "system" fn(),
    place_item_fn: unsafe extern "system" fn(),
    remove_item_fn: unsafe extern "system" fn(),
}

impl GameFns {
    fn find() -> Result<Self, Box<dyn Error>> {
        let process = unsafe { GetCurrentProcess() };
        let module = unsafe { GetModuleHandleW(null()) };

        let mut module_info = MaybeUninit::uninit();

        let success = unsafe {
            GetModuleInformation(
                process,
                module,
                module_info.as_mut_ptr(),
                size_of::<MODULEINFO>() as u32,
            )
        };

        if success == 0 {
            return Err(io::Error::last_os_error().into());
        }

        let module_info = unsafe { module_info.assume_init() };

        let module_memory = unsafe {
            slice::from_raw_parts(
                module_info.lpBaseOfDll as *const u8,
                module_info.SizeOfImage as usize,
            )
        };

        let place_block_fn_offset = memmem::find(
            module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x20, 0x4c, 0x89, 0x44, 0x24,
                0x18, 0x55,
            ],
        )
        .ok_or("")?;

        let remove_block_fn_offset = memmem::find(
            module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x6c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24,
                0x18, 0x57, 0x48, 0x83, 0xec, 0x40, 0x83, 0x7c, 0x24, 0x70, 0x00,
            ],
        )
        .ok_or("")?;

        let place_item_fn_offset = memmem::find(
            module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x6c, 0x24, 0x18, 0x48, 0x89, 0x74, 0x24,
                0x20, 0x57, 0x48, 0x83, 0xec, 0x40, 0x49, 0x8b, 0xf9,
            ],
        )
        .ok_or("")?;

        let remove_item_fn_offset = memmem::find(
            module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x08, 0x57, 0x48, 0x83, 0xec, 0x30, 0x48, 0x8b, 0xfa, 0x48,
                0x8b, 0xd9, 0x48, 0x85, 0xd2, 0x0f, 0x84, 0xe6, 0x00, 0x00, 0x00,
            ],
        )
        .ok_or("")?;

        let place_block_fn =
            unsafe { transmute(module_memory.as_ptr().add(place_block_fn_offset)) };

        let remove_block_fn =
            unsafe { transmute(module_memory.as_ptr().add(remove_block_fn_offset)) };

        let place_item_fn = unsafe { transmute(module_memory.as_ptr().add(place_item_fn_offset)) };

        let remove_item_fn =
            unsafe { transmute(module_memory.as_ptr().add(remove_item_fn_offset)) };

        Ok(Self {
            place_block_fn,
            remove_block_fn,
            place_item_fn,
            remove_item_fn,
        })
    }

    fn place_block(&self) {
        unsafe { (self.place_block_fn)() }
    }

    fn place_free_block(&self) {
        unsafe { (self.place_block_fn)() }
    }

    fn remove_block(&self) {
        unsafe { (self.remove_block_fn)() }
    }

    fn place_item(&self) {
        unsafe { (self.place_item_fn)() }
    }

    fn remove_item(&self) {
        unsafe { (self.remove_item_fn)() }
    }
}
