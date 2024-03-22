use std::{
    error::Error,
    io,
    mem::{size_of, transmute, MaybeUninit},
    ptr::null,
    slice,
};

use memchr::memmem;
use windows_sys::Win32::{
    Foundation::{CloseHandle, FALSE},
    System::{
        LibraryLoader::GetModuleHandleW,
        ProcessStatus::{GetModuleInformation, MODULEINFO},
        Threading::{
            GetCurrentProcess, GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION,
            PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
        },
    },
};

pub struct GameFns {
    place_block_fn: PlaceBlockFn,
    remove_block_fn: RemoveBlockFn,
    place_item_fn: PlaceItemFn,
    remove_item_fn: RemoveItemFn,
}

impl GameFns {
    pub fn find() -> Result<Self, Box<dyn Error>> {
        let process = unsafe { GetCurrentProcess() };
        let module = unsafe { GetModuleHandleW(null()) };

        let mut module_info = MaybeUninit::uninit();

        let success = unsafe {
            GetModuleInformation(
                process,
                module,
                module_info.as_mut_ptr(),
                size_of::<MODULEINFO>() as u32,
            )
        };

        if success == 0 {
            return Err(io::Error::last_os_error().into());
        }

        let module_info = unsafe { module_info.assume_init() };

        let module_memory = unsafe {
            slice::from_raw_parts(
                module_info.lpBaseOfDll as *const u8,
                module_info.SizeOfImage as usize,
            )
        };

        let place_block_fn_offset = memmem::find(
            module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x20, 0x4c, 0x89, 0x44, 0x24,
                0x18, 0x55,
            ],
        )
        .ok_or("")?;

        let remove_block_fn_offset = memmem::find(
            module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x6c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24,
                0x18, 0x57, 0x48, 0x83, 0xec, 0x40, 0x83, 0x7c, 0x24, 0x70, 0x00,
            ],
        )
        .ok_or("")?;

        let place_item_fn_offset = memmem::find(
            module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x6c, 0x24, 0x18, 0x48, 0x89, 0x74, 0x24,
                0x20, 0x57, 0x48, 0x83, 0xec, 0x40, 0x49, 0x8b, 0xf9,
            ],
        )
        .ok_or("")?;

        let remove_item_fn_offset = memmem::find(
            module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x08, 0x57, 0x48, 0x83, 0xec, 0x30, 0x48, 0x8b, 0xfa, 0x48,
                0x8b, 0xd9, 0x48, 0x85, 0xd2, 0x0f, 0x84, 0xe6, 0x00, 0x00, 0x00,
            ],
        )
        .ok_or("")?;

        let place_block_fn =
            unsafe { transmute(module_memory.as_ptr().add(place_block_fn_offset)) };

        let remove_block_fn =
            unsafe { transmute(module_memory.as_ptr().add(remove_block_fn_offset)) };

        let place_item_fn = unsafe { transmute(module_memory.as_ptr().add(place_item_fn_offset)) };

        let remove_item_fn =
            unsafe { transmute(module_memory.as_ptr().add(remove_item_fn_offset)) };

        Ok(Self {
            place_block_fn,
            remove_block_fn,
            place_item_fn,
            remove_item_fn,
        })
    }

    fn place_block(&self) {
        unsafe { (self.place_block_fn)() }
    }

    fn place_free_block(&self) {
        unsafe { (self.place_block_fn)() }
    }

    fn remove_block(&self) {
        unsafe { (self.remove_block_fn)() }
    }

    fn place_item(&self) {
        unsafe { (self.place_item_fn)() }
    }

    fn remove_item(&self) {
        unsafe { (self.remove_item_fn)() }
    }
}

pub fn hook_place_block() -> Result<(), Box<dyn Error>> {
    let current_process_id = unsafe { GetCurrentProcessId() };

    let current_process = unsafe {
        OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE,
            FALSE,
            current_process_id,
        )
    };

    let exe_module = unsafe { GetModuleHandleW(null()) };

    let mut exe_module_info = MaybeUninit::uninit();

    let success = unsafe {
        GetModuleInformation(
            current_process,
            exe_module,
            exe_module_info.as_mut_ptr(),
            size_of::<MODULEINFO>() as u32,
        )
    };

    if success == 0 {
        return Err(io::Error::last_os_error().into());
    }

    let exe_module_info = unsafe { exe_module_info.assume_init() };

    let exe_module_memory = unsafe {
        slice::from_raw_parts(
            exe_module_info.lpBaseOfDll as *const u8,
            exe_module_info.SizeOfImage as usize,
        )
    };

    let place_block_fn_end_offset = memmem::find(exe_module_memory, &[]).ok_or("")?;

    unsafe { CloseHandle(current_process) };

    todo!()
}

pub fn hook_remove_block() -> Result<(), Box<dyn Error>> {
    todo!()
}

pub fn hook_place_item() -> Result<(), Box<dyn Error>> {
    todo!()
}

pub fn hook_remove_item() -> Result<(), Box<dyn Error>> {
    todo!()
}

type PlaceBlockFn = unsafe extern "system" fn();

type RemoveBlockFn = unsafe extern "system" fn();

type PlaceItemFn = unsafe extern "system" fn();

type RemoveItemFn = unsafe extern "system" fn();

struct GameBlock {
    ptr: *mut u8,
}

impl GameBlock {}
