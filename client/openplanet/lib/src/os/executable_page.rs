use std::{
    ffi::c_void,
    io,
    mem::MaybeUninit,
    ptr::{null, NonNull},
    slice,
};

use windows_sys::Win32::{
    Foundation::TRUE,
    System::{
        Memory::{
            VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_DECOMMIT, MEM_RELEASE, MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        },
        SystemInformation::GetSystemInfo,
    },
};

struct ExecutablePage {
    ptr: NonNull<u8>,
    len: usize,
    capacity: usize,
}

impl ExecutablePage {
    pub fn new() -> io::Result<Self> {
        let mut system_info = MaybeUninit::uninit();

        unsafe { GetSystemInfo(system_info.as_mut_ptr()) };

        let system_info = unsafe { system_info.assume_init() };

        let page_size = system_info.dwPageSize as usize;

        let ptr = unsafe {
            VirtualAlloc(
                null(),
                page_size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            )
        };

        let ptr = NonNull::new(ptr as *mut u8).ok_or_else(io::Error::last_os_error)?;

        Ok(ExecutablePage {
            ptr,
            len: 0,
            capacity: page_size,
        })
    }

    pub fn place(&mut self, slice: &[u8]) -> Option<&[u8]> {
        if self.len + slice.len() > self.capacity {
            return None;
        }

        let mem =
            unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr().add(self.len), slice.len()) };

        mem.copy_from_slice(slice);

        self.len += slice.len();

        Some(mem)
    }
}

impl Drop for ExecutablePage {
    fn drop(&mut self) {
        let success = unsafe {
            VirtualFree(
                self.ptr.as_ptr() as *mut c_void,
                self.capacity,
                MEM_DECOMMIT | MEM_RELEASE,
            )
        };

        assert_eq!(success, TRUE);
    }
}
