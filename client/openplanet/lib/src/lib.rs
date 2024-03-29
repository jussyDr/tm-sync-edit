mod game;

use std::{ffi::c_void, panic};

use native_dialog::{MessageDialog, MessageType};
use windows_sys::Win32::{
    Foundation::{BOOL, HINSTANCE, TRUE},
    System::SystemServices::DLL_PROCESS_ATTACH,
};

const FILE_NAME: &str = "SyncEdit.dll";

#[no_mangle]
unsafe extern "system" fn DllMain(
    _module: HINSTANCE,
    call_reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    if call_reason == DLL_PROCESS_ATTACH {
        panic::set_hook(Box::new(|panic_info| {
            let _ = MessageDialog::new()
                .set_type(MessageType::Error)
                .set_title(FILE_NAME)
                .set_text(&panic_info.to_string())
                .show_alert();
        }));
    }

    TRUE
}

#[no_mangle]
unsafe extern "system" fn CreateContext() -> *mut Context {
    Box::into_raw(Box::new(Context))
}

#[no_mangle]
unsafe extern "system" fn DestroyContext(context: *mut Context) {
    drop(Box::from_raw(context));
}

struct Context;
