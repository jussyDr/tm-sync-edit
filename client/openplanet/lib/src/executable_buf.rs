use std::{
    ffi::c_void,
    ptr::{self, null, NonNull},
};

use windows_sys::Win32::{
    Foundation::FALSE,
    System::Memory::{VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_DECOMMIT, PAGE_EXECUTE_READWRITE},
};

pub struct ExecutableBuf {
    ptr: NonNull<u8>,
    len: usize,
}

impl ExecutableBuf {
    pub fn new(buf: &[u8]) -> Result<Self, ()> {
        unsafe {
            let len = buf.len();

            let ptr = NonNull::new(
                VirtualAlloc(null(), len, MEM_COMMIT, PAGE_EXECUTE_READWRITE) as *mut u8,
            )
            .ok_or(())?;

            ptr::copy_nonoverlapping(buf.as_ptr(), ptr.as_ptr(), len);

            Ok(Self { ptr, len })
        }
    }

    pub const fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }
}

impl Drop for ExecutableBuf {
    fn drop(&mut self) {
        unsafe {
            let success = VirtualFree(self.ptr.as_ptr() as *mut c_void, self.len, MEM_DECOMMIT);
            assert!(success != FALSE);
        }
    }
}
