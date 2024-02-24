#![warn(clippy::missing_docs_in_private_items)]

//! Dynamic library for the Openplanet Sync Edit client.

mod hook;
mod os;

use std::{
    mem::transmute,
    panic,
    sync::{Arc, Mutex, OnceLock},
};

use hook::{hook_end, hook_start, Hook, HookError};
use memchr::memmem;
use os::{
    executable_page::ExecutablePage,
    message_box,
    process_memory::{exe_module_memory, ProcessMemorySlice},
    DllCallReason, MessageBoxType,
};
use windows_sys::Win32::Foundation::{BOOL, TRUE};

static PLACE_BLOCK_FN: OnceLock<PlaceBlockFn> = OnceLock::new();

static STATE: Mutex<State> = Mutex::new(State {
    place_block_hook: None,
});

struct State {
    place_block_hook: Option<Hook>,
}

/// DLL entry point.
#[no_mangle]
extern "system" fn DllMain(_module: isize, call_reason: DllCallReason, _reserved: isize) -> BOOL {
    match call_reason {
        DllCallReason::ProcessAttach => {
            panic::set_hook(Box::new(|panic_info| {
                message_box(
                    &panic_info.to_string(),
                    "SyncEdit.dll",
                    MessageBoxType::Error,
                )
                .unwrap();
            }));
        }
        DllCallReason::ProcessDetach => {
            let _ = panic::take_hook();
        }
        _ => {}
    }

    TRUE
}

#[no_mangle]
extern "system" fn Init() {
    let exe_module_memory = exe_module_memory().unwrap();
    let mut executable_page = Arc::new(Mutex::new(ExecutablePage::new().unwrap()));

    let place_block_hook = hook_place_block(
        &exe_module_memory,
        Arc::clone(&executable_page),
        place_block_callback,
    )
    .unwrap();

    let remove_block_hook =
        hook_remove_block(&exe_module_memory, executable_page, remove_block_callback).unwrap();

    STATE.lock().unwrap().place_block_hook = Some(place_block_hook);

    message_box("initialized", "SyncEdit.dll", MessageBoxType::Info).unwrap();
}

#[no_mangle]
extern "system" fn Destroy() {
    STATE.lock().unwrap().place_block_hook = None;

    message_box("destroyed", "SyncEdit.dll", MessageBoxType::Info).unwrap();
}

/// Callback of the place block hook.
extern "win64" fn place_block_callback(_block: *const Block) {
    message_box("placed block", "SyncEdit.dll", MessageBoxType::Info).unwrap();
}

/// Callback of the remove block hook.
extern "win64" fn remove_block_callback(
    _editor: usize,
    _block: *const Block,
    _param_3: i32,
    _param_4: usize,
    _param_5: i32,
) {
    message_box("removed block", "SyncEdit.dll", MessageBoxType::Info).unwrap();
}

/// Corresponds to the Trackmania CGameCtnBlock class.
#[repr(C)]
struct Block;

/// Hook the place block function.
fn hook_place_block(
    exe_module_memory: &ProcessMemorySlice,
    executable_page: Arc<Mutex<ExecutablePage>>,
    callback: extern "win64" fn(*const Block),
) -> Result<Hook, HookError> {
    let code_pattern = &[
        0x4c, 0x8d, 0x9c, 0x24, 0xc0, 0x00, 0x00, 0x00, 0x49, 0x8b, 0x5b, 0x38, 0x49, 0x8b, 0x73,
        0x48, 0x49, 0x8b, 0xe3, 0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x5f, 0x5d, 0xc3,
    ];

    let offset_in_pattern = 16;

    unsafe {
        hook_end(
            exe_module_memory,
            executable_page,
            code_pattern,
            offset_in_pattern,
            callback as usize,
        )
    }
}

/// Hook the remove block function.
fn hook_remove_block(
    exe_module_memory: &ProcessMemorySlice,
    executable_page: Arc<Mutex<ExecutablePage>>,
    callback: extern "win64" fn(usize, *const Block, i32, usize, i32),
) -> Result<Hook, HookError> {
    let code_pattern = &[
        0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x6c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x18,
        0x57, 0x48, 0x83, 0xec, 0x40, 0x83, 0x7c, 0x24, 0x70, 0x00,
    ];

    unsafe {
        hook_start(
            exe_module_memory,
            executable_page,
            code_pattern,
            15,
            callback as usize,
            5,
        )
    }
}

/// Function type used when placing blocks.
type PlaceBlockFn = unsafe extern "system" fn() -> *mut Block;

/// Try to find the function used for placing blocks.
fn find_place_block(exe_module_memory: ProcessMemorySlice) -> Result<PlaceBlockFn, ()> {
    let code_pattern = [
        0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x20, 0x4c, 0x89, 0x44, 0x24, 0x18,
        0x55,
    ];

    let mem = unsafe { exe_module_memory.as_slice() };

    let off = memmem::find(mem, &code_pattern).unwrap();

    let ptr = unsafe { mem.as_ptr().add(off) };

    let func = unsafe { transmute(ptr) };

    Ok(func)
}
