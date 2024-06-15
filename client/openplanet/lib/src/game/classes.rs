use std::{
    ops::{Deref, DerefMut},
    slice, str,
};

use autopad::autopad;

pub trait Class {
    const ID: u32;
}

/// Reference-counted pointer to a `Nod` instance.
#[repr(transparent)]
pub struct NodRef<T: DerefMut<Target = Nod>> {
    ptr: *mut T,
}

impl<T: DerefMut<Target = Nod>> NodRef<T> {
    pub fn cast_mut<U: Class + DerefMut<Target = Nod>>(&mut self) -> Option<&mut NodRef<U>> {
        if self.is_instance_of::<U>() {
            unsafe { Some(&mut *(self as *mut Self as *mut NodRef<U>)) }
        } else {
            None
        }
    }
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

impl<T: DerefMut<Target = Nod>> DerefMut for NodRef<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T: DerefMut<Target = Nod>> Drop for NodRef<T> {
    fn drop(&mut self) {
        let nod = unsafe { (*self.ptr).deref_mut() };

        nod.ref_count -= 1;

        if nod.ref_count == 0 {
            unsafe { ((*nod.vtable).destructor)(nod, true) };
        }
    }
}

autopad! {
    /// CMwNod.
    #[repr(C)]
    pub struct Nod {
                vtable: *const NodVTable,
        0x10 => ref_count: u32,
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
        0x7f0 => pub switcher: NodRef<Switcher>,
        0xb18 => pub mania_title_control_script_api: NodRef<ManiaTitleControlScriptApi>,
    }
}

autopad! {
    /// CGameCtnBlockInfo.
    #[repr(C)]
    pub struct BlockInfo {
                    nod: Nod,
        0x38 => pub name: FastString,
    }
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

/// CGameCtnBlock.
pub struct Block;

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

impl Deref for Switcher {
    type Target = Nod;

    fn deref(&self) -> &Nod {
        &self.nod
    }
}

impl DerefMut for Switcher {
    fn deref_mut(&mut self) -> &mut Nod {
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
    pub yaw_pitch_roll: [f32; 3],
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
        0x270 => place_block: unsafe extern "system" fn(
            this: *mut EditorCommon,
            block_info: *const BlockInfo,
            param_3: usize,
            coord: *const [u32; 3],
            param_5: u32,
            param_6: u32,
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
            param_17: u32,
            param_18: usize,
            param_19: u32,
            param_20: u32,
        ) -> *mut Block,
    }
}

impl EditorCommon {
    pub unsafe fn place_block(&mut self, block_info: &BlockInfo) -> &mut Block {
        let coord = [20, 20, 20];

        &mut *((*(self.vtable as *const EditorCommonVTable)).place_block)(
            self, block_info, 0, &coord, 0, 0, 0, 0, 0xffffffff, 0, 1, 0, 0, 0, 0, 0, 0, 0,
            0xffffffff, 0,
        )
    }
}

impl Class for EditorCommon {
    const ID: u32 = 0x0310e000;
}

impl Deref for EditorCommon {
    type Target = Nod;

    fn deref(&self) -> &Nod {
        &self.nod
    }
}

impl DerefMut for EditorCommon {
    fn deref_mut(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

autopad! {
    /// CGameEditorPluginMap.
    #[repr(C)]
    pub struct EditorPluginMap {
                     nod: Nod,
        0x520 => pub block_infos: Array<NodRef<BlockInfo>>,
    }
}

impl Deref for EditorPluginMap {
    type Target = Nod;

    fn deref(&self) -> &Nod {
        &self.nod
    }
}

impl DerefMut for EditorPluginMap {
    fn deref_mut(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

/// CGameManiaTitleControlScriptAPI.
#[repr(C)]
pub struct ManiaTitleControlScriptApi {
    nod: Nod,
}

impl Deref for ManiaTitleControlScriptApi {
    type Target = Nod;

    fn deref(&self) -> &Nod {
        &self.nod
    }
}

impl DerefMut for ManiaTitleControlScriptApi {
    fn deref_mut(&mut self) -> &mut Nod {
        &mut self.nod
    }
}

/// CGameItemModel.
pub struct ItemModel;
