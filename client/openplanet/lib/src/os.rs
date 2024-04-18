use std::{
    ffi::c_void,
    io::{Error, Result},
    mem::{size_of, MaybeUninit},
    num::NonZeroIsize,
    ptr::{self, null, null_mut, NonNull},
    slice,
};

use windows_sys::Win32::{
    Foundation::{CloseHandle, FALSE},
    System::{
        Diagnostics::Debug::WriteProcessMemory,
        LibraryLoader::GetModuleHandleW,
        Memory::{
            VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
        },
        ProcessStatus::{GetModuleInformation, MODULEINFO},
        Threading::{
            GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION,
            PROCESS_VM_READ, PROCESS_VM_WRITE,
        },
    },
};

pub struct Process {
    handle: NonZeroIsize,
}

impl Process {
    pub fn open_current() -> Result<Self> {
        let pid = unsafe { GetCurrentProcessId() };

        let handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION
                    | PROCESS_VM_OPERATION
                    | PROCESS_VM_READ
                    | PROCESS_VM_WRITE,
                FALSE,
                pid,
            )
        };

        let handle = NonZeroIsize::new(handle).ok_or_else(Error::last_os_error)?;

        Ok(Self { handle })
    }

    pub fn exe_module_memory(&self) -> Result<&[u8]> {
        let exe_module = unsafe { GetModuleHandleW(null()) };

        let mut exe_module_info = MaybeUninit::uninit();

        let success = unsafe {
            GetModuleInformation(
                self.handle.get(),
                exe_module,
                exe_module_info.as_mut_ptr(),
                size_of::<MODULEINFO>() as u32,
            )
        };

        if success == 0 {
            return Err(Error::last_os_error());
        }

        let exe_module_info = unsafe { exe_module_info.assume_init() };

        let exe_module_memory = unsafe {
            slice::from_raw_parts(
                exe_module_info.lpBaseOfDll as *const u8,
                exe_module_info.SizeOfImage as usize,
            )
        };

        Ok(exe_module_memory)
    }

    pub unsafe fn write_memory(&self, ptr: *mut u8, bytes: &[u8]) -> Result<()> {
        let result = unsafe {
            WriteProcessMemory(
                self.handle.get(),
                ptr as *mut c_void,
                bytes.as_ptr() as *const c_void,
                bytes.len(),
                null_mut(),
            )
        };

        if result == 0 {
            return Err(Error::last_os_error());
        }

        Ok(())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let success = unsafe { CloseHandle(self.handle.get()) };

        if success == 0 {
            panic!("{}", Error::last_os_error());
        }
    }
}

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
