//! Functionality for interacting with the actual game.
//!
//! A lot of items here depend on the current game version, and can break in the future.

mod hook;

pub use hook::*;

use autopad::autopad;

autopad! {
    #[repr(C)]
    pub struct Block {
        0x60 => pub x: u32,
        0x64 => pub y: u32,
        0x68 => pub z: u32,
        0x6C => pub direction: u32,
        0x8C => pub flags: u32,
        0x9C => pub elem_color: u8
    }
}

#[repr(C)]
pub struct Item;

#[repr(C)]
pub struct ItemParams;
