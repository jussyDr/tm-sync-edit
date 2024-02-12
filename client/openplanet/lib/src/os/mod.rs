//! Safe abstractions of specific OS functionality.

pub mod executable_page;
pub mod process_memory;

use std::{
    io::{Error, Result},
    iter,
};

use windows_sys::Win32::{
    System::SystemServices::{
        DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
    },
    UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MESSAGEBOX_STYLE},
};

/// Reason why a DLL entry point is called.
#[allow(dead_code)]
#[repr(u32)]
pub enum DllCallReason {
    ProcessAttach = DLL_PROCESS_ATTACH,
    ProcessDetach = DLL_PROCESS_DETACH,
    ThreadAttach = DLL_THREAD_ATTACH,
    ThreadDetach = DLL_THREAD_DETACH,
}

/// Type of a message box.
#[repr(u32)]
pub enum MessageBoxType {
    Error = MB_ICONERROR,
    Info = MB_ICONINFORMATION,
}

/// Display a message box.
pub fn message_box(text: &str, caption: &str, ty: MessageBoxType) -> Result<()> {
    let text = encode_utf16_null_terminated(text);
    let caption = encode_utf16_null_terminated(caption);

    let result = unsafe { MessageBoxW(0, text.as_ptr(), caption.as_ptr(), ty as MESSAGEBOX_STYLE) };

    if result == 0 {
        return Err(Error::last_os_error());
    }

    Ok(())
}

/// Encode the given `string` as a null-terminated UTF-16 string.
fn encode_utf16_null_terminated(string: &str) -> Vec<u16> {
    string.encode_utf16().chain(iter::once(0)).collect()
}
