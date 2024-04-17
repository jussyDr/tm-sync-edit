//! Functionality for interacting with the actual game.
//!
//! A lot of items here depend on the current game version, and can break in the future.

mod fns;
mod hook;

pub use fns::*;
pub use hook::*;

use autopad::autopad;
use ordered_float::NotNan;
use shared::{Direction, ElemColor};

use std::{ffi::c_char, mem::MaybeUninit, ops::Deref, slice, str};

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
    struct NodVTable {
        0x018 => class_id: unsafe extern "system" fn(this: *mut Nod, class_id: *mut u32) -> *mut u32
    }
}

autopad! {
    // CMwNod.
    #[repr(C)]
    pub struct Nod {
                     vtable: *const NodVTable,
        0x018 =>     article: *mut Article,
        0x028 => pub id: u32
    }
}

impl Nod {
    pub fn article(&self) -> &Article {
        unsafe { &*self.article }
    }

    pub fn class_id(&mut self) -> u32 {
        let mut class_id = MaybeUninit::uninit();

        unsafe { ((*self.vtable).class_id)(self, class_id.as_mut_ptr()) };

        unsafe { class_id.assume_init() }
    }
}

autopad! {
    // CSystemFidFile.
    #[repr(C)]
    pub struct FidFile {
        0x018 =>     parent_folder: *mut FidsFolder,
        0x080 => pub nod: *mut Nod,
        0x0d0 =>     name: *const c_char,
        0x0d8 =>     name_len: u32
    }
}

impl FidFile {
    pub fn parent_folder(&self) -> &FidsFolder {
        unsafe { &*self.parent_folder }
    }

    pub fn name(&self) -> &str {
        let bytes =
            unsafe { slice::from_raw_parts(self.name as *const u8, self.name_len as usize) };

        unsafe { str::from_utf8_unchecked(bytes) }
    }
}

autopad! {
    // CSystemFidsFolder.
    #[repr(C)]
    pub struct FidsFolder {
        0x018 => parent_folder: *mut FidsFolder,
        0x028 => leaves: Array<FidFile>,
        0x038 => trees: Array<FidsFolder>,
        0x058 => name: CompactString
    }
}

impl FidsFolder {
    pub fn parent_folder(&self) -> Option<&FidsFolder> {
        if self.parent_folder.is_null() {
            None
        } else {
            Some(unsafe { &*self.parent_folder })
        }
    }

    pub fn leaves(&self) -> &[&FidFile] {
        self.leaves.as_slice()
    }

    pub fn trees(&self) -> &[&FidsFolder] {
        self.trees.as_slice()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

autopad! {
    // CGameCtnArticle.
    #[repr(C)]
    pub struct Article {
        0x018 =>                fid: *mut FidFile,
        0x108 => item_model_article: *mut Article
    }
}

impl Article {
    pub fn fid(&self) -> &FidFile {
        unsafe { &*self.fid }
    }

    pub fn item_model_article(&self) -> Option<&Article> {
        if self.item_model_article.is_null() {
            None
        } else {
            unsafe { Some(&*self.item_model_article) }
        }
    }
}

// CGameCtnBlockInfo.
#[repr(C)]
pub struct BlockInfo {
    nod: Nod,
}

impl Deref for BlockInfo {
    type Target = Nod;

    fn deref(&self) -> &Nod {
        &self.nod
    }
}

// CGameItemModel.
#[repr(C)]
pub struct ItemModel {
    nod: Nod,
}

impl Deref for ItemModel {
    type Target = Nod;

    fn deref(&self) -> &Nod {
        &self.nod
    }
}

autopad! {
    // CGameCtnBlock.
    #[repr(C)]
    pub struct Block {
        0x028 =>     block_info: *mut BlockInfo,
        0x060 => pub x_coord: u32,
        0x064 => pub y_coord: u32,
        0x068 => pub z_coord: u32,
        0x06C => pub direction: Direction,
        0x074 => pub x_pos: NotNan<f32>,
        0x078 => pub y_pos: NotNan<f32>,
        0x07C => pub z_pos: NotNan<f32>,
        0x080 => pub yaw: NotNan<f32>,
        0x084 => pub pitch: NotNan<f32>,
        0x088 => pub roll: NotNan<f32>,
        0x08C => pub flags: BlockFlags,
        0x09C => pub elem_color: ElemColor
    }
}

impl Block {
    pub fn block_info(&self) -> &BlockInfo {
        unsafe { &*self.block_info }
    }
}

#[repr(transparent)]
pub struct BlockFlags(u32);

impl BlockFlags {
    pub fn is_ground(&self) -> bool {
        self.0 & 0x00001000 != 0
    }

    pub fn is_ghost(&self) -> bool {
        self.0 & 0x10000000 != 0
    }

    pub fn is_free(&self) -> bool {
        self.0 & 0x20000000 != 0
    }
}

autopad! {
    /// CGameCtnAnchoredObject.
    #[repr(C)]
    pub struct Item {
        0x028 => pub params: ItemParams,
        0x158 => model: *mut ItemModel
    }
}

impl Item {
    pub fn model(&self) -> &ItemModel {
        unsafe { &*self.model }
    }
}

#[repr(C)]
pub struct ItemParams {
    pub x_coord: u32,
    pub y_coord: u32,
    pub z_coord: u32,
    pub yaw: NotNan<f32>,
    pub pitch: NotNan<f32>,
    pub roll: NotNan<f32>,
    pub param_7: u32,
    pub x_pos: NotNan<f32>,
    pub y_pos: NotNan<f32>,
    pub z_pos: NotNan<f32>,
    pub param_11: [f32; 9],
    pub param_12: [f32; 3],
    pub param_13: f32,
    pub param_14: u32,
    pub param_15: u32,
    pub param_16: [u32; 10],
    pub param_17: [f32; 3],
    pub param_18: u32,
    pub param_19: usize,
}

/// CGameCtnEditorFree.
pub struct MapEditor;
