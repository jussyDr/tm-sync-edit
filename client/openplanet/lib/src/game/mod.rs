//! Functionality for interacting with the actual game.
//!
//! A lot of items here depend on the current game version, and can break in the future.

mod fns;
mod hook;

pub use fns::*;
pub use hook::*;

use std::{ffi::c_void, ops::Deref, slice, str};

use autopad::autopad;

#[repr(C)]
struct Array<T> {
    ptr: *const *const T,
    len: u32,
    cap: u32,
}

impl<T> Array<T> {
    fn as_slice(&self) -> &[&T] {
        unsafe { slice::from_raw_parts(self.ptr as _, self.len as usize) }
    }
}

#[repr(C)]
struct CompactString {
    data: [u8; 12],
    len: u32,
}

impl CompactString {
    fn as_str(&self) -> &str {
        if self.len as usize >= self.data.len() || self.data[self.data.len() - 1] != 0 {
            let ptr = usize::from_le_bytes(self.data[..8].try_into().unwrap()) as *const u8;
            let bytes = unsafe { slice::from_raw_parts(ptr, self.len as usize) };
            str::from_utf8(bytes).expect("string is not valid UTF-8")
        } else {
            str::from_utf8(&self.data[..self.len as usize]).expect("string is not valid UTF-8")
        }
    }
}

autopad! {
    #[repr(C)]
    pub struct FidFile {
        0x080 => nod: *const u8
    }
}

impl FidFile {
    pub unsafe fn nod<T>(&self) -> Option<&T> {
        if self.nod.is_null() {
            None
        } else {
            Some(&*(self.nod as *const T))
        }
    }
}

autopad! {
    #[repr(C)]
    pub struct FidsFolder {
        0x028 => leaves: Array< FidFile>,
        0x038 => trees: Array< FidsFolder>,
        0x058 => dir_name: CompactString
    }
}

impl FidsFolder {
    pub fn leaves(&self) -> &[&FidFile] {
        self.leaves.as_slice()
    }

    pub fn trees(&self) -> &[&FidsFolder] {
        self.trees.as_slice()
    }

    pub fn dir_name(&self) -> &str {
        self.dir_name.as_str()
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
    pub struct Collector {
        0x018 => article: *mut Article,
        0x048 => name: CompactString
    }
}

impl Collector {
    pub fn article(&self) -> &Article {
        unsafe { &*self.article }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
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
        collector: Collector
    }
}

impl Deref for BlockInfo {
    type Target = Collector;

    fn deref(&self) -> &Collector {
        &self.collector
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

autopad! {
    #[repr(C)]
    pub struct ItemModel {
        collector: Collector
    }
}

impl Deref for ItemModel {
    type Target = Collector;

    fn deref(&self) -> &Collector {
        &self.collector
    }
}

pub struct Editor;
