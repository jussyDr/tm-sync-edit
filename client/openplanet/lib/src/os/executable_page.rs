//! Provides the `ExecutablePage` type.

use std::{
    ffi::c_void,
    io::{Error, ErrorKind, Result},
    mem::MaybeUninit,
    ptr::{null, NonNull},
    slice,
};

use windows_sys::Win32::System::{
    Memory::{
        VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
    },
    SystemInformation::GetSystemInfo,
};

/// A page containing executable memory.
pub struct ExecutablePage {
    /// Pointer to the start of the executable page.
    ptr: NonNull<u8>,
    /// Number of bytes that have been allocated in this page.
    allocated: usize,
    /// Size of the page.
    size: usize,
}

impl ExecutablePage {
    /// Allocate a new executable page.
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
            .map(|ptr| Self {
                ptr,
                allocated: 0,
                size,
            })
            .ok_or_else(Error::last_os_error)
    }

    /// Try to allocate a slice with the given `size` in this executable page.
    pub fn alloc(&mut self, size: usize) -> Result<&mut [u8]> {
        if self.allocated + size > self.size {
            return Err(Error::from(ErrorKind::OutOfMemory));
        }

        let slice =
            unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr().add(self.allocated), size) };

        self.allocated += size;

        Ok(slice)
    }
}

impl Drop for ExecutablePage {
    fn drop(&mut self) {
        let result = unsafe { VirtualFree(self.ptr.as_ptr() as *mut c_void, 0, MEM_RELEASE) };

        assert_ne!(result, 0, "{}", Error::last_os_error());
    }
}

// SAFETY:
// The `ptr` returned by `VirtualAlloc` is unique.
unsafe impl Send for ExecutablePage {}

// SAFETY:
// The `ptr` returned by `VirtualAlloc` is unique.
unsafe impl Sync for ExecutablePage {}
