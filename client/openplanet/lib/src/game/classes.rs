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
pub struct NodRef<T> {
    ptr: *mut T,
}

impl<T> Deref for NodRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for NodRef<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

/// CMwNod.
#[repr(C)]
pub struct Nod {
    vtable: *const NodVtable,
}

autopad! {
    #[repr(C)]
    struct NodVtable {
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

/// CGameCtnMenus.
pub struct Menus;

impl Class for Menus {
    const ID: u32 = 0x030c9000;
}

autopad! {
    /// CGameSwitcher.
    #[repr(C)]
    pub struct Switcher {
        0x20 => pub module_stack: Array<NodRef<SwitcherModule>>,
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

/// CGameCtnEditorCommon.
pub struct EditorCommon;

impl Class for EditorCommon {
    const ID: u32 = 0x0310e000;
}

/// CGameManiaTitleControlScriptAPI.
pub struct ManiaTitleControlScriptApi;
