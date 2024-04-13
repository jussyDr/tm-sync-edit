use std::{
    error::Error,
    io,
    mem::{size_of, transmute, MaybeUninit},
    ptr::null,
    slice,
};

use memchr::memmem;
use windows_sys::Win32::System::{
    LibraryLoader::GetModuleHandleW,
    ProcessStatus::{GetModuleInformation, MODULEINFO},
    Threading::GetCurrentProcess,
};

type PlaceBlockFn = unsafe extern "system" fn();

type RemoveBlockFn = unsafe extern "system" fn();

type PlaceItemFn = unsafe extern "system" fn();

type RemoveItemFn = unsafe extern "system" fn();

pub struct GameFns {
    place_block_fn: PlaceBlockFn,
    remove_block_fn: RemoveBlockFn,
    place_item_fn: PlaceItemFn,
    remove_item_fn: RemoveItemFn,
}

impl GameFns {
    pub fn find() -> Result<Self, Box<dyn Error>> {
        let current_process = unsafe { GetCurrentProcess() };

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

        let place_block_fn_offset = memmem::find(
            exe_module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x20, 0x4c, 0x89, 0x44, 0x24,
                0x18, 0x55,
            ],
        )
        .ok_or("failed to find place block fn pattern")?;

        let remove_block_fn_offset = memmem::find(
            exe_module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x6c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24,
                0x18, 0x57, 0x48, 0x83, 0xec, 0x40, 0x83, 0x7c, 0x24, 0x70, 0x00,
            ],
        )
        .ok_or("failed to find remove block fn pattern")?;

        let place_item_fn_offset = memmem::find(
            exe_module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x6c, 0x24, 0x18, 0x48, 0x89, 0x74, 0x24,
                0x20, 0x57, 0x48, 0x83, 0xec, 0x40, 0x49, 0x8b, 0xf9,
            ],
        )
        .ok_or("failed to find place item fn pattern")?;

        let remove_item_fn_offset = memmem::find(
            exe_module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x08, 0x57, 0x48, 0x83, 0xec, 0x30, 0x48, 0x8b, 0xfa, 0x48,
                0x8b, 0xd9, 0x48, 0x85, 0xd2, 0x0f, 0x84, 0xe6, 0x00, 0x00, 0x00,
            ],
        )
        .ok_or("failed to find remove item fn pattern")?;

        let place_block_fn =
            unsafe { transmute(exe_module_memory.as_ptr().add(place_block_fn_offset)) };

        let remove_block_fn =
            unsafe { transmute(exe_module_memory.as_ptr().add(remove_block_fn_offset)) };

        let place_item_fn =
            unsafe { transmute(exe_module_memory.as_ptr().add(place_item_fn_offset)) };

        let remove_item_fn =
            unsafe { transmute(exe_module_memory.as_ptr().add(remove_item_fn_offset)) };

        Ok(Self {
            place_block_fn,
            remove_block_fn,
            place_item_fn,
            remove_item_fn,
        })
    }
}
