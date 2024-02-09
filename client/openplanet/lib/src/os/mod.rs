//! Safe abstractions over specific OS functionality.

pub mod executable_page;

use std::{
    ffi::OsString,
    io,
    mem::{size_of, MaybeUninit},
    num::NonZeroIsize,
    os::windows::ffi::OsStringExt,
    path::PathBuf,
    ptr::null,
    slice,
};

use windows_sys::Win32::{
    Foundation::FALSE,
    System::{
        LibraryLoader::{GetModuleFileNameW, GetModuleHandleW},
        ProcessStatus::{GetModuleInformation, MODULEINFO},
        Threading::{
            GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION,
            PROCESS_VM_READ, PROCESS_VM_WRITE,
        },
    },
    UI::WindowsAndMessaging::{MessageBoxW, MESSAGEBOX_STYLE},
};

use crate::encode_utf16_null_terminated;

pub fn message_box(
    text: impl AsRef<str>,
    caption: impl AsRef<str>,
    style: MESSAGEBOX_STYLE,
) -> io::Result<()> {
    let text = encode_utf16_null_terminated(text);
    let caption = encode_utf16_null_terminated(caption);

    let result = unsafe { MessageBoxW(0, text.as_ptr(), caption.as_ptr(), style) };

    if result == 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ModuleHandle(NonZeroIsize);

impl ModuleHandle {
    pub fn to_exe_file() -> io::Result<Self> {
        let module_handle = unsafe { GetModuleHandleW(null()) };

        NonZeroIsize::new(module_handle)
            .map(Self)
            .ok_or_else(io::Error::last_os_error)
    }

    pub fn get_file_name(&self) -> io::Result<PathBuf> {
        let mut file_name_buf = vec![0; 128];

        let result = unsafe {
            GetModuleFileNameW(
                self.0.get(),
                file_name_buf.as_mut_ptr(),
                file_name_buf.len() as u32,
            )
        };

        if result == 0 {
            return Err(io::Error::last_os_error());
        }

        let file_name = PathBuf::from(OsString::from_wide(&file_name_buf[..result as usize]));

        Ok(file_name)
    }
}

pub struct ProcessHandle(NonZeroIsize);

impl ProcessHandle {
    pub fn open_current() -> io::Result<Self> {
        let process_id = unsafe { GetCurrentProcessId() };

        let process_handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION
                    | PROCESS_VM_OPERATION
                    | PROCESS_VM_READ
                    | PROCESS_VM_WRITE,
                FALSE,
                process_id,
            )
        };

        NonZeroIsize::new(process_handle)
            .map(Self)
            .ok_or_else(io::Error::last_os_error)
    }

    pub fn module_memory(&self, module: ModuleHandle) -> io::Result<&[u8]> {
        let mut module_information = MaybeUninit::uninit();

        let result = unsafe {
            GetModuleInformation(
                self.0.get(),
                module.0.get(),
                module_information.as_mut_ptr(),
                size_of::<MODULEINFO>() as u32,
            )
        };

        if result == 0 {
            return Err(io::Error::last_os_error());
        }

        let module_information = unsafe { module_information.assume_init() };

        let module_memory = unsafe {
            slice::from_raw_parts(
                module_information.lpBaseOfDll as *const u8,
                module_information.SizeOfImage as usize,
            )
        };

        Ok(module_memory)
    }
}
