use std::ops::{Deref, DerefMut};

use autopad::autopad;
use gamebox::engines::game::map::{Direction, ElemColor, PhaseOffset};
use ordered_float::NotNan;

use crate::game::Class;

use super::{FidFile, ItemModel, Nod, PackDesc};

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

autopad! {
    // CGameCtnBlockInfo.
    #[repr(C)]
    pub struct BlockInfo {
                    nod: Nod,
        0x28 => pub id: u32
    }
}

impl Class for BlockInfo {
    const ID: u32 = 0x0304e000;
}

impl Deref for BlockInfo {
    type Target = Nod;

    fn deref(&self) -> &Nod {
        &self.nod
    }
}

impl DerefMut for BlockInfo {
    fn deref_mut(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

autopad! {
    // CGameCtnBlock.
    #[repr(C)]
    pub struct Block {
                    nod: Nod,
        0x28 =>     block_info: *mut BlockInfo,
        0x60 => pub x_coord: u32,
        0x64 => pub y_coord: u32,
        0x68 => pub z_coord: u32,
        0x6c => pub direction: Direction,
        0x74 => pub x_pos: NotNan<f32>,
        0x78 => pub y_pos: NotNan<f32>,
        0x7c => pub z_pos: NotNan<f32>,
        0x80 => pub yaw: NotNan<f32>,
        0x84 => pub pitch: NotNan<f32>,
        0x88 => pub roll: NotNan<f32>,
        0x8c => pub flags: u32,
        0x9c => pub elem_color: ElemColor
    }
}

impl Block {
    pub fn block_info(&self) -> &BlockInfo {
        unsafe { &*self.block_info }
    }

    pub fn is_ground(&self) -> bool {
        self.flags & 0x00001000 != 0
    }

    pub fn is_ghost(&self) -> bool {
        self.flags & 0x10000000 != 0
    }

    pub fn is_free(&self) -> bool {
        self.flags & 0x20000000 != 0
    }
}

impl Deref for Block {
    type Target = Nod;

    fn deref(&self) -> &Nod {
        &self.nod
    }
}

impl DerefMut for Block {
    fn deref_mut(&mut self) -> &mut Nod {
        &mut self.nod
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
    pub pivot_pos: [NotNan<f32>; 3],
    pub param_13: f32,
    pub param_14: u32,
    pub param_15: u32,
    pub param_16: u32,
    pub parent_block: *const *const Block,
    pub skin: *const PackDesc,
    pub skin_effect: *const PackDesc,
    pub param_20: [u32; 3],
    pub param_21: [f32; 3],
    pub elem_color: ElemColor,
    pub anim_offset: PhaseOffset,
}

autopad! {
    /// CGameCtnAnchoredObject.
    #[repr(C)]
    pub struct Item {
                     nod: Nod,
        0x028 => pub params: ItemParams,
        0x158 =>     model: *mut ItemModel
    }
}

impl Item {
    pub fn model(&self) -> &ItemModel {
        unsafe { &*self.model }
    }
}

impl Deref for Item {
    type Target = Nod;

    fn deref(&self) -> &Nod {
        &self.nod
    }
}

impl DerefMut for Item {
    fn deref_mut(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

/// CGameCtnEditorFree.
pub struct MapEditor;

impl Class for MapEditor {
    const ID: u32 = 0x0310f000;
}
