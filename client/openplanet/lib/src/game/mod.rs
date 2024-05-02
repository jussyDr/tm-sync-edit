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

use std::{
    ffi::c_char,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    path::PathBuf,
    slice, str,
};

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
        let bytes = if self.len as usize >= self.data.len() || self.data[self.data.len() - 1] != 0 {
            let ptr = usize::from_le_bytes(self.data[..8].try_into().unwrap()) as *const u8;
            unsafe { slice::from_raw_parts(ptr, self.len as usize) }
        } else {
            &self.data[..self.len as usize]
        };

        str::from_utf8(bytes).expect("string is not valid UTF-8")
    }
}

autopad! {
    // CMwNod.
    #[repr(C)]
    pub struct Nod {
                    vtable: *const NodVtable,
        0x08 =>     file: *const FidFile,
        0x10 =>     ref_count: u32,
        0x18 =>     article: *mut Article,
        0x28 => pub id: u32
    }
}

autopad! {
    #[repr(C)]
    struct NodVtable {
        0x08 => destructor: unsafe extern "system" fn(this: *mut Nod, should_free: bool) -> *mut Nod,
        0x18 => class_id: unsafe extern "system" fn(this: *const Nod, class_id: *mut u32) -> *mut u32,
        0x20 => is_instance_of: unsafe extern "system" fn(this: *const Nod, class_id: u32) -> bool,
    }
}

impl Nod {
    pub fn file(&self) -> &FidFile {
        unsafe { &*self.file }
    }

    pub fn article(&self) -> &Article {
        unsafe { &*self.article }
    }

    pub fn class_id(&self) -> u32 {
        let mut class_id: MaybeUninit<u32> = MaybeUninit::uninit();

        unsafe { ((*self.vtable).class_id)(self, class_id.as_mut_ptr()) };

        unsafe { class_id.assume_init() }
    }

    pub fn is_instance_of(&self, class_id: u32) -> bool {
        unsafe { ((*self.vtable).is_instance_of)(self, class_id) }
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
    #[repr(C)]
    struct FidsFolderVtable {
        0xf8 => update_tree: unsafe extern "system" fn(this: *mut FidsFolder, recurse: bool),
    }
}

autopad! {
    // CSystemFidsFolder.
    #[repr(C)]
    pub struct FidsFolder {
                vtable: *const FidsFolderVtable,
        0x18 => parent_folder: *mut FidsFolder,
        0x28 => leaves: Array<FidFile>,
        0x38 => trees: Array<FidsFolder>,
        0x58 => name: CompactString
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

    pub fn update_tree(&mut self, recurse: bool) {
        unsafe { ((*self.vtable).update_tree)(self, recurse) }
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
    // CGameItemModel.
    #[repr(C)]
    pub struct ItemModel {
                 nod: Nod,
        0x288 => entity_model: *const Nod,
    }
}

impl ItemModel {
    pub fn entity_model(&self) -> Option<&Nod> {
        if self.entity_model.is_null() {
            None
        } else {
            unsafe { Some(&*self.entity_model) }
        }
    }
}

impl Class for ItemModel {
    const ID: u32 = 0x2e002000;
}

impl Deref for ItemModel {
    type Target = Nod;

    fn deref(&self) -> &Nod {
        &self.nod
    }
}

impl DerefMut for ItemModel {
    fn deref_mut(&mut self) -> &mut Nod {
        &mut self.nod
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
        0x08C => pub flags: u32,
        0x09C => pub elem_color: ElemColor
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

pub trait Class {
    const ID: u32;
}

pub fn cast_nod<T: Class>(nod: &Nod) -> Option<&T> {
    if nod.is_instance_of(T::ID) {
        unsafe { Some(&*(nod as *const _ as *const _)) }
    } else {
        None
    }
}

/// Reference-counted pointer to a [Nod].
pub struct NodRef<T: DerefMut<Target = Nod>> {
    ptr: *mut T,
}

impl<T: DerefMut<Target = Nod>> Clone for NodRef<T> {
    fn clone(&self) -> Self {
        let nod = unsafe { (*self.ptr).deref_mut() };

        nod.ref_count += 1;

        Self { ptr: self.ptr }
    }
}

impl<T: DerefMut<Target = Nod>> Deref for NodRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T: DerefMut<Target = Nod>> Drop for NodRef<T> {
    fn drop(&mut self) {
        let nod = unsafe { (*self.ptr).deref_mut() };

        nod.ref_count -= 1;

        if nod.ref_count == 0 {
            unsafe { ((*(nod.vtable)).destructor)(nod, true) };
        }
    }
}

impl<T: DerefMut<Target = Nod>> From<&T> for NodRef<T> {
    fn from(x: &T) -> Self {
        let ptr = x as *const T as *mut T;
        let nod = unsafe { (*ptr).deref_mut() };

        nod.ref_count += 1;

        Self { ptr }
    }
}

pub fn fids_folder_full_path(folder: &FidsFolder) -> PathBuf {
    if let Some(parent_folder) = folder.parent_folder() {
        let mut path = fids_folder_full_path(parent_folder);
        path.push(folder.name());

        path
    } else {
        PathBuf::from(folder.name())
    }
}
