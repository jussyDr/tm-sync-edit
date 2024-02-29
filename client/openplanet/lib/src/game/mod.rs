use std::{
    io::{Error, ErrorKind, Result},
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

const PLACE_BLOCK_PATTERN: &[u8] = &[
    0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x20, 0x4c, 0x89, 0x44, 0x24, 0x18, 0x55,
];

const REMOVE_BLOCK_PATTERN: &[u8] = &[
    0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x6c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x18, 0x57,
    0x48, 0x83, 0xec, 0x40, 0x83, 0x7c, 0x24, 0x70, 0x00,
];

const PLACE_ITEM_PATTERN: &[u8] = &[
    0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x6c, 0x24, 0x18, 0x48, 0x89, 0x74, 0x24, 0x20, 0x57,
    0x48, 0x83, 0xec, 0x40, 0x49, 0x8b, 0xf9,
];

const REMOVE_ITEM_PATTERN: &[u8] = &[
    0x48, 0x89, 0x5c, 0x24, 0x08, 0x57, 0x48, 0x83, 0xec, 0x30, 0x48, 0x8b, 0xfa, 0x48, 0x8b, 0xd9,
    0x48, 0x85, 0xd2, 0x0f, 0x84, 0xe6, 0x00, 0x00, 0x00,
];

type PlaceBlockFn = unsafe extern "system" fn(
    editor: usize,
    block_info: usize,
    param_3: u64,
    coord: *const Coord,
    dir: u32,
    elem_color: u8,
    param_7: u8,
    param_8: u32,
    param_9: i32,
    is_ghost_1: u32,
    param_11: u32,
    param_12: u32,
    is_ground: u32,
    param_14: u32,
    is_ghost_2: u32,
    param_16: usize,
    is_free: u32,
    free_transform: *const Transform,
    param_19: u32,
    param_20: u32,
);

type RemoveBlockFn = unsafe extern "system" fn();

type PlaceItemFn = unsafe extern "system" fn(usize, usize, usize, usize) -> i32;

type RemoveItemFn = unsafe extern "system" fn();

#[repr(C)]
pub struct Coord {
    x: i32,
    y: i32,
    z: i32,
}

#[repr(C)]
pub struct Transform {
    x: f32,
    y: f32,
    z: f32,
    yaw: f32,
    pitch: f32,
    roll: f32,
}

#[repr(u32)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

#[repr(u8)]
pub enum ElemColor {
    Default,
    White,
    Green,
    Blue,
    Red,
    Black,
}

pub struct Fns {
    place_block_fn: PlaceBlockFn,
    remove_block_fn: RemoveBlockFn,
    place_item_fn: PlaceItemFn,
    remove_item_fn: RemoveItemFn,
}

impl Fns {
    pub unsafe fn find() -> Result<Self> {
        let current_process = unsafe { GetCurrentProcess() };

        let game_module = unsafe { GetModuleHandleW(null()) };

        if game_module == 0 {
            return Err(Error::last_os_error());
        }

        let mut game_module_info = MaybeUninit::uninit();

        let result = unsafe {
            GetModuleInformation(
                current_process,
                game_module,
                game_module_info.as_mut_ptr(),
                size_of::<MODULEINFO>() as u32,
            )
        };

        if result == 0 {
            return Err(Error::last_os_error());
        }

        let game_module_info = unsafe { game_module_info.assume_init() };

        let game_module_memory_ptr = game_module_info.lpBaseOfDll as *const u8;

        let game_module_memory = unsafe {
            slice::from_raw_parts(
                game_module_memory_ptr,
                game_module_info.SizeOfImage as usize,
            )
        };

        let place_block_offset = memmem::find(game_module_memory, PLACE_BLOCK_PATTERN)
            .ok_or(Error::from(ErrorKind::Other))?;

        let remove_block_offset = memmem::find(game_module_memory, REMOVE_BLOCK_PATTERN)
            .ok_or(Error::from(ErrorKind::Other))?;

        let place_item_offset = memmem::find(game_module_memory, PLACE_ITEM_PATTERN)
            .ok_or(Error::from(ErrorKind::Other))?;

        let remove_item_offset = memmem::find(game_module_memory, REMOVE_ITEM_PATTERN)
            .ok_or(Error::from(ErrorKind::Other))?;

        let place_block_fn = unsafe { transmute(game_module_memory_ptr.add(place_block_offset)) };
        let remove_block_fn = unsafe { transmute(game_module_memory_ptr.add(remove_block_offset)) };
        let place_item_fn = unsafe { transmute(game_module_memory_ptr.add(place_item_offset)) };
        let remove_item_fn = unsafe { transmute(game_module_memory_ptr.add(remove_item_offset)) };

        Ok(Self {
            place_block_fn,
            remove_block_fn,
            place_item_fn,
            remove_item_fn,
        })
    }

    unsafe fn place_block_inner(
        &self,
        editor: usize,
        block_info: usize,
        coord: *const Coord,
        dir: u32,
        elem_color: u8,
        param_9: i32,
        is_ghost: u32,
        is_not_free: u32,
        is_ground: u32,
        is_free: u32,
        free_transform: *const Transform,
    ) {
        (self.place_block_fn)(
            editor,
            block_info,
            0,
            coord,
            dir,
            elem_color,
            0,
            0,
            param_9,
            is_ghost,
            is_not_free,
            0,
            is_ground,
            0,
            is_ghost,
            0,
            is_free,
            free_transform,
            0xffffffff,
            0,
        );
    }

    pub unsafe fn place_block(
        &self,
        editor: usize,
        block_info: usize,
        x: i32,
        y: i32,
        z: i32,
        dir: Direction,
        elem_color: ElemColor,
        is_ghost: bool,
        is_ground: bool,
    ) {
        let coord = Coord { x, y, z };

        self.place_block_inner(
            editor,
            block_info,
            &coord,
            dir as u32,
            elem_color as u8,
            -1,
            if is_ghost { 1 } else { 0 },
            1,
            if is_ground { 1 } else { 0 },
            0,
            null(),
        );
    }

    pub unsafe fn place_free_block(
        &self,
        editor: usize,
        block_info: usize,
        elem_color: ElemColor,
        x: f32,
        y: f32,
        z: f32,
        yaw: f32,
        pitch: f32,
        roll: f32,
    ) {
        let transform = Transform {
            x,
            y,
            z,
            yaw,
            pitch,
            roll,
        };

        self.place_block_inner(
            editor,
            block_info,
            &Coord { x: -1, y: 0, z: -1 },
            0,
            elem_color as u8,
            0x3f,
            0,
            0,
            0,
            1,
            &transform,
        );
    }
}
