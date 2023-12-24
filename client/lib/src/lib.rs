use std::{ffi::c_void, iter, panic};

use windows_sys::Win32::{
    Foundation::{BOOL, HINSTANCE, TRUE},
    System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MESSAGEBOX_STYLE},
};

#[no_mangle]
extern "system" fn DllMain(_instance: HINSTANCE, call_reason: u32, _reserved: *mut c_void) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            panic::set_hook(Box::new(|info| {
                message_box(&format!("{info}"), "Panic!", MB_ICONERROR).unwrap();
            }));
        }
        DLL_PROCESS_DETACH => {
            let _ = panic::take_hook();
        }
        _ => {}
    }

    TRUE
}

#[no_mangle]
extern "C" fn Run() {
    message_box("Test", "We are running!", MB_ICONINFORMATION).unwrap();
}

fn message_box(text: &str, caption: &str, ty: MESSAGEBOX_STYLE) -> Result<(), ()> {
    unsafe {
        let text = encode_utf16(text);
        let caption = encode_utf16(caption);

        let result = MessageBoxW(0, text.as_ptr(), caption.as_ptr(), ty);

        if result == 0 {
            return Err(());
        }

        Ok(())
    }
}

fn encode_utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(iter::once(0)).collect()
}
