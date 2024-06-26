//! Game classes.

use std::{
    ops::{Deref, DerefMut},
    ptr::null,
    slice, str,
};

use autopad::autopad;
use gamebox::{
    engines::game::map::{Direction, ElemColor, YawPitchRoll},
    Vec3,
};

pub trait Class {
    const ID: u32;
}

pub trait Inherits {
    type Parent;

    fn parent(&mut self) -> &mut Self::Parent;
}

/// Reference-counted pointer to a `Nod` instance.
#[repr(transparent)]
pub struct NodRef<T: Inherits<Parent = Nod>> {
    ptr: *mut T,
}

impl<T: Inherits<Parent = Nod>> NodRef<T> {
    pub unsafe fn from_ptr(ptr: *mut T) -> Self {
        Self { ptr }
    }

    pub fn cast_mut<U: Class + Inherits<Parent = Nod>>(&mut self) -> Option<&mut NodRef<U>> {
        if self.parent().is_instance_of::<U>() {
            unsafe { Some(&mut *(self as *mut Self as *mut NodRef<U>)) }
        } else {
            None
        }
    }
}

impl<T: Inherits<Parent = Nod>> Clone for NodRef<T> {
    fn clone(&self) -> Self {
        let nod = unsafe { (*self.ptr).parent() };

        nod.ref_count += 1;

        Self { ptr: self.ptr }
    }
}

impl<T: Inherits<Parent = Nod>> Deref for NodRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T: Inherits<Parent = Nod>> DerefMut for NodRef<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T: Inherits<Parent = Nod>> Drop for NodRef<T> {
    fn drop(&mut self) {
        // let nod = unsafe { (*self.ptr).parent() };

        // nod.ref_count -= 1;

        // if nod.ref_count == 0 {
        //     unsafe { ((*nod.vtable).destructor)(nod, true) };
        // }
    }
}

autopad! {
    /// CMwNod.
    #[repr(C)]
    pub struct Nod {
                    vtable: *const NodVTable,
        0x08 => pub fid_file: NodRef<FidFile>,
        0x10 =>     ref_count: u32,
    }
}

autopad! {
    #[repr(C)]
    struct NodVTable {
        0x08 => destructor: unsafe extern "system" fn(this: *mut Nod, free_memory: bool),
        0x20 => is_instance_of: unsafe extern "system" fn(this: *const Nod, class_id: u32) -> bool,
    }
}

impl Nod {
    pub fn is_instance_of<T: Class>(&self) -> bool {
        unsafe { ((*self.vtable).is_instance_of)(self, T::ID) }
    }
}

impl Inherits for Nod {
    type Parent = Nod;

    fn parent(&mut self) -> &mut Nod {
        self
    }
}

#[repr(C)]
pub struct FastString {
    union: FastStringUnion,
    is_ptr: bool,
    len: u32,
}

impl Deref for FastString {
    type Target = str;

    fn deref(&self) -> &str {
        if self.is_ptr {
            let bytes = unsafe { slice::from_raw_parts(self.union.ptr, self.len as usize) };
            unsafe { str::from_utf8_unchecked(bytes) }
        } else {
            let bytes = unsafe { &self.union.chars[..self.len as usize] };
            unsafe { str::from_utf8_unchecked(bytes) }
        }
    }
}

#[repr(packed)]
union FastStringUnion {
    chars: [u8; 11],
    ptr: *const u8,
}

#[repr(C)]
pub struct ConstString {
    ptr: *const u8,
    len: u32,
}

impl Deref for ConstString {
    type Target = str;

    fn deref(&self) -> &str {
        let bytes = unsafe { slice::from_raw_parts(self.ptr, self.len as usize) };
        unsafe { str::from_utf8_unchecked(bytes) }
    }
}

/// CMwSArray.
#[repr(C)]
pub struct Array<T> {
    ptr: *mut T,
    len: u32,
    cap: u32,
}

impl<T> Deref for Array<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr, self.len as usize) }
    }
}

impl<T> DerefMut for Array<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.len as usize) }
    }
}

autopad! {
    /// CGameManiaPlanet.
    #[repr(C)]
    pub struct ManiaPlanet {
                     nod: Nod,
        0x7f0 => pub switcher: NodRef<Switcher>,
        0xb18 => pub mania_title_control_script_api: NodRef<ManiaTitleControlScriptApi>,
    }
}

impl Deref for ManiaPlanet {
    type Target = Nod;

    fn deref(&self) -> &Nod {
        &self.nod
    }
}

impl DerefMut for ManiaPlanet {
    fn deref_mut(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

impl Inherits for ManiaPlanet {
    type Parent = Nod;

    fn parent(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

/// CGameCtnBlockInfo.
#[repr(C)]
pub struct BlockInfo {
    collector: Collector,
}

impl Class for BlockInfo {
    const ID: u32 = 0x0304e000;
}

impl Deref for BlockInfo {
    type Target = Collector;

    fn deref(&self) -> &Collector {
        &self.collector
    }
}

impl Inherits for BlockInfo {
    type Parent = Nod;

    fn parent(&mut self) -> &mut Nod {
        &mut self.collector.nod
    }
}

/// CGameCtnBlock.
#[repr(C)]
pub struct Block {
    nod: Nod,
}

impl Inherits for Block {
    type Parent = Nod;

    fn parent(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

/// CGameCtnMenus.
pub struct Menus;

impl Class for Menus {
    const ID: u32 = 0x030c9000;
}

autopad! {
    /// CGameSwitcher.
    #[repr(C)]
    pub struct Switcher {
                    nod: Nod,
        0x20 => pub module_stack: Array<NodRef<SwitcherModule>>,
    }
}

impl Inherits for Switcher {
    type Parent = Nod;

    fn parent(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

autopad! {
    /// CGameCtnAnchoredObject.
    #[repr(C)]
    pub struct Item {
        0x28 => pub params: ItemParams,
    }
}

#[repr(C)]
pub struct ItemParams {
    pub coord: [u32; 3],
    pub rotation: [f32; 3],
    pub param_3: u32,
    pub pos: [f32; 3],
    pub param_5: [f32; 9],
    pub pivot_pos: [f32; 3],
    pub param_7: f32,
    pub param_8: u32,
    pub param_9: u32,
    pub param_10: u32,
    pub param_11: u32,
    pub param_12: u32,
    pub param_13: u32,
    pub param_14: u32,
    pub param_15: u32,
    pub param_16: u32,
    pub param_17: u32,
    pub param_18: u32,
    pub param_19: u32,
    pub param_20: [f32; 3],
    pub elem_color: u8,
    pub anim_offset: u8,
    pub param_22: u32,
}

/// CGameSwitcherModule.
#[repr(C)]
pub struct SwitcherModule {
    nod: Nod,
}

impl Deref for SwitcherModule {
    type Target = Nod;

    fn deref(&self) -> &Nod {
        &self.nod
    }
}

impl DerefMut for SwitcherModule {
    fn deref_mut(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

impl Inherits for SwitcherModule {
    type Parent = Nod;

    fn parent(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

autopad! {
    /// CGameCtnEditorCommon.
    #[repr(C)]
    pub struct EditorCommon {
                     nod: Nod,
        0xbdc => pub air_mode: bool,
        0xfb0 => pub plugin_map_type: NodRef<EditorPluginMap>,
    }
}

autopad! {
    #[repr(C)]
    struct EditorCommonVTable {
        0x268 => can_place_block: unsafe extern "system" fn(
            this: *mut EditorCommon,
            block_info: *const BlockInfo,
            coord: *const [u32; 3],
            dir: u32,
            param_5: usize,
            param_6: usize,
            param_7: usize,
            param_8: usize,
            param_9: usize,
            param_10: usize,
            param_11: u32,
            param_12: usize,
            param_13: usize,
            param_14: u32,
            param_15: u32,
            param_16: u32,
            param_17: u32,
            param_18: u32,
            param_19: u32
        ) -> bool,
        #[allow(clippy::type_complexity)]
        0x270 => place_block: unsafe extern "system" fn(
            this: *mut EditorCommon,
            block_info: *const BlockInfo,
            param_3: usize,
            coord: *const [u32; 3],
            dir: u32,
            elem_color: u8,
            param_7: u8,
            param_8: u32,
            param_9: u32,
            param_10: u32,
            param_11: u32,
            param_12: u32,
            param_13: u32,
            param_14: u32,
            param_15: u32,
            param_16: usize,
            is_free: u32,
            transform: *const [f32; 6],
            param_19: u32,
            param_20: u32,
        ) -> *mut Block,
        0x2c8 => remove_all: unsafe extern "system" fn(this: *mut EditorCommon, param_2: u32)
    }
}

impl EditorCommon {
    pub fn can_place_block(
        &mut self,
        block_info: &BlockInfo,
        coord: Vec3<u8>,
        dir: Direction,
    ) -> bool {
        let coord = [coord.x as u32, coord.y as u32, coord.z as u32];

        unsafe {
            ((*(self.nod.vtable as *const EditorCommonVTable)).can_place_block)(
                self, block_info, &coord, dir as u32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0xffffffff, 0,
            )
        }
    }

    pub fn place_ghost_block(
        &mut self,
        block_info: &BlockInfo,
        coord: Vec3<u8>,
        dir: Direction,
        elem_color: ElemColor,
    ) -> Option<NodRef<Block>> {
        if self.can_place_block(block_info, coord, dir) {
            let coord = [coord.x as u32, coord.y as u32, coord.z as u32];

            let block = unsafe {
                ((*(self.nod.vtable as *const EditorCommonVTable)).place_block)(
                    self,
                    block_info,
                    0,
                    &coord,
                    dir as u32,
                    elem_color as u8,
                    0,
                    0,
                    0xffffffff,
                    1,
                    1,
                    0,
                    0,
                    0xffffffff,
                    1,
                    0,
                    0,
                    null(),
                    0xffffffff,
                    0,
                )
            };

            if block.is_null() {
                None
            } else {
                unsafe { Some(NodRef::from_ptr(block)) }
            }
        } else {
            None
        }
    }

    pub fn place_free_block(
        &mut self,
        block_info: &BlockInfo,
        pos: Vec3<f32>,
        rotation: YawPitchRoll,
        elem_color: ElemColor,
    ) -> Option<NodRef<Block>> {
        let coord = [0xffffffff, 0, 0xffffffff];

        let transform = [
            pos.x,
            pos.y,
            pos.z,
            rotation.yaw,
            rotation.pitch,
            rotation.roll,
        ];

        let block = unsafe {
            ((*(self.nod.vtable as *const EditorCommonVTable)).place_block)(
                self,
                block_info,
                0,
                &coord,
                0,
                elem_color as u8,
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
                &transform,
                0xffffffff,
                0,
            )
        };

        if block.is_null() {
            None
        } else {
            unsafe { Some(NodRef::from_ptr(block)) }
        }
    }

    pub fn remove_all(&mut self) {
        unsafe { ((*(self.nod.vtable as *const EditorCommonVTable)).remove_all)(self, 0) };
    }
}

impl Class for EditorCommon {
    const ID: u32 = 0x0310e000;
}

impl Inherits for EditorCommon {
    type Parent = Nod;

    fn parent(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

/// CGameEditorPluginMap.
#[repr(C)]
pub struct EditorPluginMap {
    nod: Nod,
}

impl Inherits for EditorPluginMap {
    type Parent = Nod;

    fn parent(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

/// CGameManiaTitleControlScriptAPI.
#[repr(C)]
pub struct ManiaTitleControlScriptApi {
    nod: Nod,
}

impl Inherits for ManiaTitleControlScriptApi {
    type Parent = Nod;

    fn parent(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

autopad! {
    /// CSystemFidsFolder.
    #[repr(C)]
    pub struct FidsFolder {
                    nod: Nod,
        0x28 => pub leaves: Array<NodRef<FidFile>>,
        0x38 => pub trees: Array<NodRef<FidsFolder>>,
        0x58 => pub path: FastString
    }
}

autopad! {
    #[repr(C)]
    struct FidsFolderVTable {
        0xf8 => update_tree: unsafe extern "system" fn(this: *mut FidsFolder, recurse: bool),
    }
}

impl FidsFolder {
    pub fn update_tree(&mut self, recurse: bool) {
        unsafe { ((*(self.nod.vtable as *const FidsFolderVTable)).update_tree)(self, recurse) };
    }
}

impl Inherits for FidsFolder {
    type Parent = Nod;

    fn parent(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

autopad! {
    /// CSystemFidFile.
    #[repr(C)]
    pub struct FidFile {
                    nod: Nod,
        0x18 => pub parent_folder: NodRef<FidsFolder>,
        0xd0 => pub name: ConstString,
    }
}

impl Inherits for FidFile {
    type Parent = Nod;

    fn parent(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

autopad! {
    /// CGameCtnCollector.
    #[repr(C)]
    pub struct Collector {
                    nod: Nod,
        0x38 => pub name: FastString,
    }
}

autopad! {
    /// CGameItemModel.
    #[repr(C)]
    pub struct ItemModel {
                     collector: Collector,
        0x288 => pub entity_model: Option<NodRef<Nod>>
    }
}

impl Class for ItemModel {
    const ID: u32 = 0x2e002000;
}

impl Deref for ItemModel {
    type Target = Collector;

    fn deref(&self) -> &Collector {
        &self.collector
    }
}

impl Inherits for ItemModel {
    type Parent = Nod;

    fn parent(&mut self) -> &mut Nod {
        &mut self.collector.nod
    }
}
