mod hook;

pub use hook::*;

#[repr(C)]
pub struct Block {
    pad_1: [u8; 96],
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub direction: u32,
}

#[repr(C)]
pub struct Item;

#[repr(C)]
pub struct ItemParams;
