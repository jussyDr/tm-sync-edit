use std::{
    io::{Error, Result},
    iter,
};

use windows_sys::Win32::{
    System::SystemServices::{
        DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
    },
    UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION},
};

#[allow(dead_code)]
#[repr(u32)]
pub enum DllCallReason {
    ProcessAttach = DLL_PROCESS_ATTACH,
    ProcessDetach = DLL_PROCESS_DETACH,
    ThreadAttach = DLL_THREAD_ATTACH,
    ThreadDetach = DLL_THREAD_DETACH,
}

#[repr(u32)]
pub enum MessageBoxType {
    Error = MB_ICONERROR,
    Info = MB_ICONINFORMATION,
}

pub fn message_box(caption: &str, text: &str, ty: MessageBoxType) -> Result<()> {
    let caption = encode_utf16_null_terminated(caption);
    let text = encode_utf16_null_terminated(text);

    let result = unsafe { MessageBoxW(0, text.as_ptr(), caption.as_ptr(), ty as u32) };

    if result == 0 {
        return Err(Error::last_os_error());
    }

    Ok(())
}

fn encode_utf16_null_terminated(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(iter::once(0)).collect()
}
