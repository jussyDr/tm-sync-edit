use std::{
    io,
    mem::{size_of, MaybeUninit},
    num::{NonZeroIsize, ParseIntError},
    ptr::null,
    slice,
};

use windows_sys::Win32::{
    Foundation::{CloseHandle, FALSE},
    System::{
        LibraryLoader::GetModuleHandleW,
        ProcessStatus::{GetModuleInformation, MODULEINFO},
        Threading::{GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    },
};

/// An open handle to a process.
pub struct Process {
    handle: NonZeroIsize,
}

impl Process {
    /// Open a handle to the current process.
    pub fn open_current() -> io::Result<Self> {
        let current_process_id = unsafe { GetCurrentProcessId() };

        let current_process_handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                FALSE,
                current_process_id,
            )
        };

        NonZeroIsize::new(current_process_handle)
            .map(|handle| Self { handle })
            .ok_or_else(io::Error::last_os_error)
    }

    /// Obtain the memory of the main process module.
    pub fn main_module_memory(&self) -> io::Result<ModuleMemory> {
        let main_module = unsafe { GetModuleHandleW(null()) };

        if main_module == 0 {
            return Err(io::Error::last_os_error());
        }

        let mut main_module_info = MaybeUninit::uninit();

        let success = unsafe {
            GetModuleInformation(
                self.handle.get(),
                main_module,
                main_module_info.as_mut_ptr(),
                size_of::<MODULEINFO>() as u32,
            )
        };

        if success == 0 {
            return Err(io::Error::last_os_error());
        }

        let main_module_info = unsafe { main_module_info.assume_init() };

        let slice = unsafe {
            slice::from_raw_parts(
                main_module_info.lpBaseOfDll as *const u8,
                main_module_info.SizeOfImage as usize,
            )
        };

        Ok(ModuleMemory { slice })
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let success = unsafe { CloseHandle(self.handle.get()) };

        if success == 0 {
            panic!("{}", io::Error::last_os_error());
        }
    }
}

/// Memory of a process module.
pub struct ModuleMemory {
    slice: &'static [u8],
}

impl ModuleMemory {
    /// Try to find a specific byte pattern in this module's memory.
    pub fn find_pattern(&self, pattern: &str) -> Result<Option<*const u8>, ParseIntError> {
        let tokens = pattern
            .split_ascii_whitespace()
            .map(|token| {
                if token == "??" {
                    Ok(None)
                } else {
                    Ok(Some(u8::from_str_radix(token, 16)?))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        for offset in 0..self.slice.len() - tokens.len() {
            let mut matches = true;

            for (i, token) in tokens.iter().copied().enumerate() {
                if let Some(value) = token {
                    if self.slice[offset + i] != value {
                        matches = false;
                        break;
                    }
                }
            }

            if matches {
                return unsafe { Ok(Some(self.slice.as_ptr().add(offset))) };
            }
        }

        Ok(None)
    }
}
