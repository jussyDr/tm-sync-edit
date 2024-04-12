//! Functionality for interacting with the actual game.
//!
//! A lot of items here depend on the current game version, and can break in the future.

mod hook;

pub use hook::*;

use std::{
    borrow::Cow,
    ffi::{c_char, c_void, CStr},
    slice,
};

use autopad::autopad;

autopad! {
    #[repr(C)]
    pub struct FidFile {}
}

autopad! {
    #[repr(C)]
    pub struct FidsFolder {
        0x028 => leaves: *mut FidFile,
        0x030 => leaves_len: u32,
        0x038 => trees: *mut FidsFolder,
        0x040 => trees_len: u32,
        0x060 => dir_name: *const c_char,
    }
}

impl FidsFolder {
    pub fn leaves(&self) -> &[FidFile] {
        unsafe { slice::from_raw_parts(self.leaves, self.leaves_len as usize) }
    }

    pub fn trees(&self) -> &[FidsFolder] {
        unsafe { slice::from_raw_parts(self.trees, self.trees_len as usize) }
    }

    pub fn dir_name(&self) -> Cow<str> {
        unsafe { CStr::from_ptr(self.dir_name) }.to_string_lossy()
    }
}

autopad! {
    #[repr(C)]
    pub struct Article {
        0x080 => pub loaded_nod: *mut c_void,
        0x108 =>     item_model_article: *mut Article
    }
}

impl Article {
    pub fn item_model_article(&self) -> Option<&Article> {
        if self.item_model_article.is_null() {
            None
        } else {
            unsafe { Some(&*self.item_model_article) }
        }
    }
}

autopad! {
    #[repr(C)]
    pub struct Block {
        0x028 =>     block_info: *mut BlockInfo,
        0x060 => pub x_coord: u32,
        0x064 => pub y_coord: u32,
        0x068 => pub z_coord: u32,
        0x06C => pub direction: u32,
        0x074 => pub x_pos: f32,
        0x078 => pub y_pos: f32,
        0x07C => pub z_pos: f32,
        0x080 => pub yaw: f32,
        0x084 => pub pitch: f32,
        0x088 => pub roll: f32,
        0x08C => pub flags: u32,
        0x09C => pub elem_color: u8
    }
}

impl Block {
    pub fn block_info(&self) -> &BlockInfo {
        unsafe { &*self.block_info }
    }
}

autopad! {
    #[repr(C)]
    pub struct BlockInfo {
        0x018 => article: *mut Article
    }
}

impl BlockInfo {
    pub fn article(&self) -> &Article {
        unsafe { &*self.article }
    }
}

autopad! {
    #[repr(C)]
    pub struct Item {}
}

autopad! {
    #[repr(C)]
    pub struct ItemParams {}
}
