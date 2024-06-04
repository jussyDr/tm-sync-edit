use std::{
    ffi::c_char,
    ops::{Deref, DerefMut},
    slice,
};

use autopad::autopad;

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
    pub fn is_instance_of(&self, class_id: u32) -> bool {
        unsafe { ((*self.vtable).is_instance_of)(self, class_id) }
    }
}

/// CMwSArray.
#[repr(C)]
pub struct Array<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
}

impl<T> Deref for Array<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}

/// CMwSConstString.
#[repr(C)]
struct ConstString {
    ptr: *const c_char,
    len: u32,
    cap: u32,
}

impl From<&str> for ConstString {
    fn from(s: &str) -> Self {
        Self {
            ptr: s.as_ptr() as *const c_char,
            len: s.len() as u32,
            cap: 0,
        }
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
    /// CGameSwitcher.
    #[repr(C)]
    pub struct Switcher {
        0x20 => pub module_stack: Array<NodRef<Nod>>,
    }
}

/// CGameCtnEditorCommon.
pub struct EditorCommon;

/// CGameManiaTitleControlScriptAPI.
pub struct ManiaTitleControlScriptApi;
