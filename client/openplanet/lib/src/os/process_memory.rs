//! Types for reading and writing process memory.

use std::{
    ffi::c_void,
    io::{Error, ErrorKind, Result},
    mem::{size_of, MaybeUninit},
    num::NonZeroIsize,
    ops::Range,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    ptr::{null, NonNull},
    slice,
};

use windows_sys::Win32::{
    Foundation::FALSE,
    System::{
        LibraryLoader::GetModuleHandleW,
        Memory::{VirtualProtectEx, PAGE_PROTECTION_FLAGS, PAGE_READWRITE},
        ProcessStatus::{GetModuleInformation, MODULEINFO},
        Threading::{
            GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION,
            PROCESS_VM_READ,
        },
    },
};

/// Obtain a memory slice to the executable module.
pub fn exe_module_memory() -> Result<ProcessMemorySlice> {
    let current_process_id = unsafe { GetCurrentProcessId() };

    let raw_current_process_handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION | PROCESS_VM_READ,
            FALSE,
            current_process_id,
        )
    };

    let current_process_handle =
        unsafe { OwnedHandle::from_raw_handle(raw_current_process_handle as *mut c_void) };

    let raw_exe_module_handle = unsafe { GetModuleHandleW(null()) };

    let exe_module_handle =
        NonZeroIsize::new(raw_exe_module_handle).ok_or_else(Error::last_os_error)?;

    let mut exe_module_info = MaybeUninit::uninit();

    let result = unsafe {
        GetModuleInformation(
            current_process_handle.as_raw_handle() as isize,
            exe_module_handle.get(),
            exe_module_info.as_mut_ptr(),
            size_of::<MODULEINFO>() as u32,
        )
    };

    if result == 0 {
        return Err(Error::last_os_error());
    }

    let exe_module_info = unsafe { exe_module_info.assume_init() };

    let ptr = unsafe { NonNull::new_unchecked(exe_module_info.lpBaseOfDll as *mut u8) };
    let len = exe_module_info.SizeOfImage as usize;

    Ok(ProcessMemorySlice {
        process_handle: current_process_handle,
        ptr,
        len,
    })
}

pub struct ProcessMemorySlice {
    process_handle: OwnedHandle,
    ptr: NonNull<u8>,
    /// Length of the slice.
    len: usize,
}

impl ProcessMemorySlice {
    pub unsafe fn as_slice(&self) -> &[u8] {
        slice::from_raw_parts(self.ptr.as_ptr(), self.len)
    }

    pub unsafe fn write(&self, bytes: &[u8]) -> Result<()> {
        if bytes.len() != self.len {
            return Err(Error::from(ErrorKind::Other));
        }

        let old_protect = virtual_protect(
            self.process_handle.as_raw_handle() as isize,
            self.ptr.as_ptr(),
            bytes.len(),
            PAGE_READWRITE,
        )?;

        let slice = slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len);

        slice.copy_from_slice(bytes);

        virtual_protect(
            self.process_handle.as_raw_handle() as isize,
            self.ptr.as_ptr(),
            bytes.len(),
            old_protect,
        )?;

        Ok(())
    }

    pub fn slice(&self, range: Range<usize>) -> Result<Self> {
        if range.end > self.len {
            return Err(Error::from(ErrorKind::Other));
        }

        let process_handle = self.process_handle.try_clone()?;
        let ptr = unsafe { NonNull::new_unchecked(self.ptr.as_ptr().add(range.start)) };
        let len = range.len();

        Ok(Self {
            process_handle,
            ptr,
            len,
        })
    }
}

unsafe impl Send for ProcessMemorySlice {}

unsafe impl Sync for ProcessMemorySlice {}

/// Change the protection of a region of pages.
unsafe fn virtual_protect(
    process_handle: isize,
    address: *const u8,
    size: usize,
    new_protect: PAGE_PROTECTION_FLAGS,
) -> Result<PAGE_PROTECTION_FLAGS> {
    let mut old_protect = MaybeUninit::uninit();

    let result = VirtualProtectEx(
        process_handle,
        address as *const c_void,
        size,
        new_protect,
        old_protect.as_mut_ptr(),
    );

    if result == 0 {
        return Err(Error::last_os_error());
    }

    Ok(old_protect.assume_init())
}
