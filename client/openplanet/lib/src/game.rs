use std::{
    ffi::c_char,
    mem::transmute,
    ops::Deref,
    ptr::{null, null_mut},
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

pub unsafe fn edit_new_map(api: &ManiaTitleControlScriptApi) {
    let ptr = transmute::<usize, unsafe extern "system" fn(*mut ManiaTitleControlScriptApi, *mut u8)>(
        0x7ff7d5fa70b0,
    );

    let mut p1: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut p2: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut p3: [u8; 4] = [0, 0, 0, 0];
    let mut p4: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut p5: [u8; 16] = [
        0x43, 0x61, 0x72, 0x53, 0x70, 0x6f, 0x72, 0x74, 0, 0, 0, 0, 8, 0, 0, 0,
    ];
    let mut p6: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let mut p7_str: [u8; 22] = [
        0x34, 0x38, 0x78, 0x34, 0x38, 0x53, 0x63, 0x72, 0x65, 0x65, 0x6e, 0x31, 0x35, 0x35, 0x44,
        0x61, 0x79, 0, 0, 0, 0, 0,
    ];

    let mut p7: Vec<u8> = vec![];
    p7.extend((p7_str.as_mut_ptr() as usize).to_le_bytes());
    p7.extend([0, 0, 0, 1, 0x11, 0, 0, 0]);

    let mut p8: [u8; 16] = [
        0x53, 0x74, 0x61, 0x64, 0x69, 0x75, 0x6d, 0, 0, 0, 0, 0, 7, 0, 0, 0,
    ];

    // 00000279158F3150

    let mut extra_params: Vec<u8> = vec![];
    extra_params.extend((p4.as_mut_ptr() as usize).to_le_bytes());
    extra_params.extend([0x7e, 0, 0, 0, 0, 0, 0, 0]);
    extra_params.extend((p5.as_mut_ptr() as usize).to_le_bytes());
    extra_params.extend([0x7e, 0, 0, 0, 0, 0, 0, 0]);
    extra_params.extend((p6.as_mut_ptr() as usize).to_le_bytes());
    extra_params.extend([0x7e, 0, 0, 0, 0, 0, 0, 0]);
    extra_params.extend((p7.as_mut_ptr() as usize).to_le_bytes());
    extra_params.extend([0x7d, 0, 0, 0, 0, 0, 0, 0]);
    extra_params.extend((p8.as_mut_ptr() as usize).to_le_bytes());
    extra_params.extend([0x7d, 0, 0, 0, 0, 0, 0, 0]);

    let mut params = vec![];
    params.extend(&[1, 9, 0, 0, 0x7d, 0x7e, 0x71, 0]);
    params.extend(0x7ff7d6eec038usize.to_le_bytes());
    params.extend((p1.as_mut_ptr() as usize).to_le_bytes());
    params.extend((p2.as_mut_ptr() as usize).to_le_bytes());
    params.extend((p3.as_mut_ptr() as usize).to_le_bytes());
    params.extend((extra_params.as_mut_ptr() as usize).to_le_bytes());

    ptr(api as *const _ as *mut _, params.as_mut_ptr());

    // 00000278EF2657A0
}
