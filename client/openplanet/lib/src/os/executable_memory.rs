use std::{
    ffi::c_void,
    io::{Error, Result},
    ptr::{self, null, NonNull},
};

use windows_sys::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
};

/// A slice of executable memory.
pub struct ExecutableMemory {
    ptr: NonNull<u8>,
}

impl ExecutableMemory {
    pub fn new(bytes: &[u8]) -> Result<Self> {
        let ptr = unsafe {
            VirtualAlloc(
                null(),
                bytes.len(),
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            ) as *mut u8
        };

        let ptr = NonNull::new(ptr).ok_or_else(Error::last_os_error)?;

        unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.as_ptr(), bytes.len()) };

        Ok(Self { ptr })
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }
}

impl Drop for ExecutableMemory {
    fn drop(&mut self) {
        let success = unsafe { VirtualFree(self.ptr.as_ptr() as *mut c_void, 0, MEM_RELEASE) };

        if success == 0 {
            panic!("{}", Error::last_os_error());
        }
    }
}
