use std::{marker::PhantomData, mem::transmute};

use crate::process::ModuleMemory;

use super::{ManiaPlanet, ManiaTitleControlScriptApi};

#[repr(C)]
struct ScriptString<'a> {
    union: ScriptStringUnion<'a>,
    flags: u32,
    len: u32,
}

impl<'a> From<&'a str> for ScriptString<'a> {
    fn from(s: &str) -> Self {
        if s.len() <= 8 {
            let mut chars = [0; 8];

            chars[..s.len()].copy_from_slice(s.as_bytes());

            Self {
                union: ScriptStringUnion { chars },
                flags: 0,
                len: s.len() as u32,
            }
        } else {
            Self {
                union: ScriptStringUnion { ptr: s.as_ptr() },
                flags: 0x01000000,
                len: s.len() as u32,
            }
        }
    }
}

#[repr(C)]
union ScriptStringUnion<'a> {
    chars: [u8; 8],
    ptr: *const u8,
    marker: PhantomData<&'a ()>,
}

type EditNewMap2FnTy =
    unsafe extern "system" fn(this: *mut ManiaTitleControlScriptApi, args: *mut u8);

pub struct EditNewMap2Fn(EditNewMap2FnTy);

impl EditNewMap2Fn {
    pub fn find(main_module_memory: &ModuleMemory) -> Option<Self> {
        let pattern = "66 0f 7f 44 24 70 e8 ?? ?? ?? ?? 8b 4d 2c";

        let ptr = unsafe { main_module_memory.find_pattern(pattern).unwrap()?.sub(754) };

        let f = unsafe { transmute::<*const u8, EditNewMap2FnTy>(ptr) };

        Some(Self(f))
    }

    pub unsafe fn call(&self, this: &mut ManiaTitleControlScriptApi, player_model: &str) {
        let mut arg_1 = ScriptString::from("Stadium");

        let mut arg_2 = ScriptString::from("48x48Screen155Day");

        let mut arg_3 = ScriptString::from("");

        let mut arg_4 = ScriptString::from(player_model);

        let mut arg_5 = ScriptString::from("");

        let mut arg_6: u32 = 0;

        let mut arg_7 = ScriptString::from("");

        let mut arg_8 = ScriptString::from("");

        let mut extra_args: Vec<u8> = vec![];
        extra_args.extend(((&mut arg_5 as *mut _) as usize).to_le_bytes());
        extra_args.extend([0x7e, 0, 0, 0, 0, 0, 0, 0]);
        extra_args.extend(((&mut arg_4 as *mut _) as usize).to_le_bytes());
        extra_args.extend([0x7e, 0, 0, 0, 0, 0, 0, 0]);
        extra_args.extend(((&mut arg_3 as *mut _) as usize).to_le_bytes());
        extra_args.extend([0x7e, 0, 0, 0, 0, 0, 0, 0]);
        extra_args.extend(((&mut arg_2 as *mut _) as usize).to_le_bytes());
        extra_args.extend([0x7d, 0, 0, 0, 0, 0, 0, 0]);
        extra_args.extend(((&mut arg_1 as *mut _) as usize).to_le_bytes());
        extra_args.extend([0x7d, 0, 0, 0, 0, 0, 0, 0]);

        let mut args = vec![];
        args.extend(&[1, 9, 0, 0, 0x7d, 0x7e, 0x71, 0]);
        args.extend(0usize.to_le_bytes());
        args.extend(((&mut arg_8 as *mut _) as usize).to_le_bytes());
        args.extend(((&mut arg_7 as *mut _) as usize).to_le_bytes());
        args.extend(((&mut arg_6 as *mut _) as usize).to_le_bytes());
        args.extend((extra_args.as_mut_ptr() as usize).to_le_bytes());

        (self.0)(this, args.as_mut_ptr());
    }
}

type BackToMainMenuFnTy = unsafe extern "system" fn(this: *mut ManiaPlanet);

pub struct BackToMainMenuFn(BackToMainMenuFnTy);

impl BackToMainMenuFn {
    pub fn find(main_module_memory: &ModuleMemory) -> Option<Self> {
        let pattern = "48 8b 89 40 03 00 00 33 d2 e9 ?? ?? ?? ??";

        let ptr = main_module_memory.find_pattern(pattern).unwrap()?;

        let f = unsafe { transmute::<*const u8, BackToMainMenuFnTy>(ptr) };

        Some(Self(f))
    }

    pub unsafe fn call(&self, this: &mut ManiaPlanet) {
        (self.0)(this);
    }
}
