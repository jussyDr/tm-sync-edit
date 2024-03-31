//! Functionality for interacting with the actual game.
//!
//! A lot of items here depend on the current game version, and can break in the future.

mod hook;

pub use hook::*;

use autopad::autopad;

autopad! {
    #[repr(C)]
    pub struct Block {
        0x28 => pub block_info: *const BlockInfo,
        0x60 => pub x_coord: u32,
        0x64 => pub y_coord: u32,
        0x68 => pub z_coord: u32,
        0x6C => pub direction: u32,
        0x74 => pub x_pos: f32,
        0x78 => pub y_pos: f32,
        0x7C => pub z_pos: f32,
        0x80 => pub yaw: f32,
        0x84 => pub pitch: f32,
        0x88 => pub roll: f32,
        0x8C => pub flags: u32,
        0x9C => pub elem_color: u8
    }
}

autopad! {
    #[repr(C)]
    pub struct BlockInfo {}
}

autopad! {
    #[repr(C)]
    pub struct Item {}
}

autopad! {
    #[repr(C)]
    pub struct ItemParams {}
}
