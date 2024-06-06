use std::{
    ops::{Deref, DerefMut},
    slice,
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
                vtable: *const NodVtable,
        0x10 => ref_count: u32,
    }
}

autopad! {
    #[repr(C)]
    struct NodVtable {
        0x08 => destructor: unsafe extern "system" fn(this: *mut Nod, free_memory: bool),
        0x20 => is_instance_of: unsafe extern "system" fn(this: *const Nod, class_id: u32) -> bool,
    }
}

impl Nod {
    pub fn is_instance_of<T: Class>(&self) -> bool {
        unsafe { ((*self.vtable).is_instance_of)(self, T::ID) }
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

autopad! {
    /// CGameManiaPlanet.
    #[repr(C)]
    pub struct ManiaPlanet {
        0x7f0 => pub switcher: NodRef<Switcher>,
        0xb18 => pub mania_title_control_script_api: NodRef<ManiaTitleControlScriptApi>,
    }
}

/// CGameCtnBlockInfo.
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

impl DerefMut for BlockInfo {
    fn deref_mut(&mut self) -> &mut Nod {
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
        0xfb0 => pub plugin_map_type: NodRef<EditorPluginMap>,
    }
}

impl Class for EditorCommon {
    const ID: u32 = 0x0310e000;
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
