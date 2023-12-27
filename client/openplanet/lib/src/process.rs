use std::{
    ffi::c_void,
    mem::{size_of, MaybeUninit},
    num::NonZeroIsize,
    ops::{Bound, RangeBounds},
    ptr::{null, null_mut},
    slice,
};

use windows_sys::Win32::{
    Foundation::FALSE,
    System::{
        Diagnostics::Debug::WriteProcessMemory,
        LibraryLoader::GetModuleHandleW,
        Memory::{VirtualProtectEx, PAGE_EXECUTE_READWRITE},
        ProcessStatus::{GetModuleInformation, MODULEINFO},
        Threading::{
            GetCurrentProcessId, OpenProcess, PROCESS_VM_OPERATION, PROCESS_VM_READ,
            PROCESS_VM_WRITE,
        },
    },
};

pub struct Process {
    handle: NonZeroIsize,
    memory_ptr: *const u8,
    memory_len: usize,
}

impl Process {
    pub fn open_current() -> Result<Self, ()> {
        unsafe {
            let current_process_id = GetCurrentProcessId();

            let handle = OpenProcess(
                PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE,
                FALSE,
                current_process_id,
            );

            let handle = NonZeroIsize::new(handle).ok_or(())?;

            let module = GetModuleHandleW(null());

            let module = NonZeroIsize::new(module).ok_or(())?;

            let mut module_info = MaybeUninit::uninit();

            let success = GetModuleInformation(
                handle.get(),
                module.get(),
                module_info.as_mut_ptr(),
                size_of::<MODULEINFO>() as u32,
            );

            if success == FALSE {
                return Err(());
            }

            let module_info = module_info.assume_init();

            Ok(Self {
                handle,
                memory_ptr: module_info.lpBaseOfDll as *const u8,
                memory_len: module_info.SizeOfImage as usize,
            })
        }
    }

    pub fn memory(&self) -> ProcessMemory {
        unsafe {
            ProcessMemory {
                handle: self.handle,
                slice: slice::from_raw_parts(self.memory_ptr, self.memory_len),
            }
        }
    }
}

pub struct ProcessMemory<'a> {
    handle: NonZeroIsize,
    slice: &'a [u8],
}

impl ProcessMemory<'_> {
    pub const unsafe fn as_slice(&self) -> &[u8] {
        self.slice
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> ProcessMemory {
        let start = match range.start_bound() {
            Bound::Included(&bound) => bound,
            Bound::Excluded(&bound) => bound + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(&bound) => bound + 1,
            Bound::Excluded(&bound) => bound,
            Bound::Unbounded => self.slice.len(),
        };

        ProcessMemory {
            handle: self.handle,
            slice: &self.slice[start..end],
        }
    }

    pub unsafe fn write(&mut self, buf: &[u8]) -> Result<(), ()> {
        let len = buf.len();
        let memory = &self.slice[..len];

        let old_protect = protect(self.handle, memory, PAGE_EXECUTE_READWRITE)?;

        let success = WriteProcessMemory(
            self.handle.get(),
            memory.as_ptr() as *const c_void,
            buf.as_ptr() as *const c_void,
            len,
            null_mut(),
        );

        protect(self.handle, memory, old_protect)?;

        if success == FALSE {
            return Err(());
        }

        Ok(())
    }
}

fn protect(handle: NonZeroIsize, memory: &[u8], new_protect: u32) -> Result<u32, ()> {
    unsafe {
        let mut old_protect = MaybeUninit::uninit();

        let success = VirtualProtectEx(
            handle.get(),
            memory.as_ptr() as *const c_void,
            memory.len(),
            new_protect,
            old_protect.as_mut_ptr(),
        );

        if success == FALSE {
            return Err(());
        }

        Ok(old_protect.assume_init())
    }
}
