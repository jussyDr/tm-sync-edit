#![warn(clippy::missing_docs_in_private_items)]

//! Dynamic library for the Openplanet Sync Edit client.

mod hook;
mod os;

use std::{
    mem::transmute,
    panic,
    sync::{Arc, Mutex},
};

use hook::{hook_return, Hook, HookError};
use os::{
    executable_page::ExecutablePage,
    message_box,
    process_memory::{exe_module_memory, ProcessMemorySlice},
    DllCallReason, MessageBoxType,
};
use windows_sys::Win32::Foundation::{BOOL, TRUE};

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

    let place_block_hook =
        hook_place_block(exe_module_memory, Arc::clone(&executable_page)).unwrap();

    STATE.lock().unwrap().place_block_hook = Some(place_block_hook);

    message_box("initialized", "SyncEdit.dll", MessageBoxType::Info).unwrap();
}

#[no_mangle]
extern "system" fn Destroy() {
    STATE.lock().unwrap().place_block_hook = None;

    message_box("destroyed", "SyncEdit.dll", MessageBoxType::Info).unwrap();
}

/// Callback of the place block hook.
extern "system" fn place_block_callback(_block: *const Block) {
    message_box("placed block", "SyncEdit.dll", MessageBoxType::Info).unwrap();
}

/// Corresponds to the Trackmania CGameCtnBlock class.
#[repr(C)]
struct Block;

/// Hook the place block function.
fn hook_place_block(
    exe_module_memory: ProcessMemorySlice,
    executable_page: Arc<Mutex<ExecutablePage>>,
) -> Result<Hook, HookError> {
    let callback: extern "system" fn(*const Block) = place_block_callback;
    let callback = unsafe { transmute(callback) };

    let code_pattern = &[
        0x4c, 0x8d, 0x9c, 0x24, 0xc0, 0x00, 0x00, 0x00, 0x49, 0x8b, 0x5b, 0x38, 0x49, 0x8b, 0x73,
        0x48, 0x49, 0x8b, 0xe3, 0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x5f, 0x5d, 0xc3,
    ];

    let offset_in_pattern = 16;

    unsafe {
        hook_return(
            exe_module_memory,
            executable_page,
            code_pattern,
            offset_in_pattern,
            callback,
        )
    }
}
