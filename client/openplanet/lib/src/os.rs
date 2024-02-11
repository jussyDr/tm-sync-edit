use core::slice;
use std::{
    ffi::c_void,
    io::{Error, ErrorKind, Result},
    iter,
    mem::MaybeUninit,
    ptr::{null, NonNull},
};

use windows_sys::Win32::{
    System::{
        Memory::{
            VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_DECOMMIT, MEM_RELEASE, MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        },
        SystemInformation::GetSystemInfo,
        SystemServices::{
            DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
        },
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

pub struct ExecutablePage {
    ptr: NonNull<u8>,
    head: usize,
    size: usize,
}

impl ExecutablePage {
    pub fn new() -> Result<Self> {
        let mut system_info = MaybeUninit::uninit();

        unsafe { GetSystemInfo(system_info.as_mut_ptr()) };

        let system_info = unsafe { system_info.assume_init() };

        let size = system_info.dwPageSize as usize;

        let ptr = unsafe {
            VirtualAlloc(
                null(),
                size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            )
        };

        NonNull::new(ptr as *mut u8)
            .map(|ptr| Self { ptr, head: 0, size })
            .ok_or_else(Error::last_os_error)
    }

    pub fn alloc(&mut self, slice: &[u8]) -> Result<&[u8]> {
        if self.head + slice.len() > self.size {
            return Err(Error::from(ErrorKind::OutOfMemory));
        }

        let executable_slice =
            unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr().add(self.head), slice.len()) };

        self.head += slice.len();

        executable_slice.copy_from_slice(slice);

        Ok(executable_slice)
    }
}

impl Drop for ExecutablePage {
    fn drop(&mut self) {
        let result = unsafe {
            VirtualFree(
                self.ptr.as_ptr() as *mut c_void,
                self.size,
                MEM_DECOMMIT | MEM_RELEASE,
            )
        };

        assert_ne!(result, 0, "{}", Error::last_os_error());
    }
}

unsafe impl Send for ExecutablePage {}

unsafe impl Sync for ExecutablePage {}
