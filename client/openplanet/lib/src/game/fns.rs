use std::{
    error::Error,
    io,
    mem::{size_of, transmute, MaybeUninit},
    ptr::{null, null_mut},
    slice,
};

use memchr::memmem;
use windows_sys::Win32::System::{
    LibraryLoader::GetModuleHandleW,
    ProcessStatus::{GetModuleInformation, MODULEINFO},
    Threading::GetCurrentProcess,
};

use super::{Block, BlockInfo, Editor};

type PlaceBlockFn = unsafe extern "system" fn(
    editor: *mut Editor,
    block_info: *mut BlockInfo,
    param_3: usize,
    coord: *mut [u32; 3],
    dir: u32,
    elem_color: u8,
    param_7: u8,
    param_8: u32,
    param_9: u32,
    is_ghost: u32,
    param_11: u32,
    param_12: u32,
    is_ground: u32,
    param_14: u32,
    is_ghost: u32,
    param_16: usize,
    is_free: u32,
    transform: *mut [f32; 6],
    param_19: u32,
    param_20: u32,
) -> *mut Block;

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

        let _ = native_dialog::MessageDialog::new()
            .set_type(native_dialog::MessageType::Error)
            .set_title("watawt")
            .set_text(&format!("{:p}", place_block_fn))
            .show_alert();

        Ok(Self {
            place_block_fn,
            remove_block_fn,
            place_item_fn,
            remove_item_fn,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn place_block(
        &self,
        editor: &mut Editor,
        block_info: &mut BlockInfo,
        x: u8,
        y: u8,
        z: u8,
        dir: u32,
        elem_color: u8,
        is_ghost: u32,
        is_ground: u32,
    ) -> Option<&Block> {
        let mut coord = [x as u32, y as u32, z as u32];

        let block = unsafe {
            (self.place_block_fn)(
                editor,
                block_info,
                0,
                &mut coord,
                dir,
                elem_color,
                0,
                0,
                0xffffffff,
                is_ghost,
                1,
                0,
                is_ground,
                0,
                is_ghost,
                0,
                0,
                null_mut(),
                0xffffffff,
                0,
            )
        };

        if block.is_null() {
            None
        } else {
            unsafe { Some(&*block) }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn place_free_block(
        &self,
        editor: &mut Editor,
        block_info: &mut BlockInfo,
        elem_color: u8,
        x: f32,
        y: f32,
        z: f32,
        yaw: f32,
        pitch: f32,
        roll: f32,
    ) -> Option<&Block> {
        let mut coord = [0xffffffff, 0, 0xffffffff];

        let mut transform = [x, y, z, yaw, pitch, roll];

        let block = unsafe {
            (self.place_block_fn)(
                editor,
                block_info,
                0,
                &mut coord,
                0,
                elem_color,
                0,
                0,
                0x3f,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                1,
                &mut transform,
                0xffffffff,
                0,
            )
        };

        if block.is_null() {
            None
        } else {
            unsafe { Some(&*block) }
        }
    }

    pub fn remove_block(&self) {}

    pub fn place_item(&self) {}

    pub fn remove_item(&self) {}
}
