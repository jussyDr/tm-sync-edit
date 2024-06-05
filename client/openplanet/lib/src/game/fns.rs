use std::{marker::PhantomData, mem::transmute};

use crate::process::ModuleMemory;

use super::{BlockInfo, EditorCommon, ManiaPlanet, ManiaTitleControlScriptApi};

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

type EditNewMap2FnType =
    unsafe extern "system" fn(this: *mut ManiaTitleControlScriptApi, args: *mut u8);

pub struct EditNewMap2Fn(EditNewMap2FnType);

impl EditNewMap2Fn {
    pub fn find(main_module_memory: &ModuleMemory) -> Option<Self> {
        let pattern = "66 0f 7f 44 24 70 e8 ?? ?? ?? ?? 8b 4d 2c";

        let ptr = unsafe { main_module_memory.find_pattern(pattern).unwrap()?.sub(754) };

        let f = unsafe { transmute::<*const u8, EditNewMap2FnType>(ptr) };

        Some(Self(f))
    }

    pub unsafe fn call(
        &self,
        this: &mut ManiaTitleControlScriptApi,
        decoration: &str,
        player_model: &str,
    ) {
        let mut arg_1 = ScriptString::from("Stadium");

        let mut arg_2 = ScriptString::from(decoration);

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

type BackToMainMenuFnType = unsafe extern "system" fn(this: *mut ManiaPlanet);

pub struct BackToMainMenuFn(BackToMainMenuFnType);

impl BackToMainMenuFn {
    pub fn find(main_module_memory: &ModuleMemory) -> Option<Self> {
        let pattern = "48 8b 89 40 03 00 00 33 d2 e9 ?? ?? ?? ??";

        let ptr = main_module_memory.find_pattern(pattern).unwrap()?;

        let f = unsafe { transmute::<*const u8, BackToMainMenuFnType>(ptr) };

        Some(Self(f))
    }

    pub unsafe fn call(&self, this: &mut ManiaPlanet) {
        (self.0)(this);
    }
}

type PlaceBlockFnType = unsafe extern "system" fn(
    this: *mut EditorCommon,
    block_info: *mut BlockInfo,
    coord: *mut u32,
    dir: u32,
    param_5: u8,
    param_6: u8,
    param_7: u32,
    param_8: u32,
    param_9: u32,
    param_10: u32,
    param_11: u32,
    param_12: u32,
    param_13: u32,
    param_14: u32,
    param_15: u32,
    param_16: u32,
    param_17: u32,
    param_18: usize,
    param_19: u32,
) -> usize;

pub struct PlaceBlockFn(PlaceBlockFnType);

impl PlaceBlockFn {
    pub fn find(main_module_memory: &ModuleMemory) -> Option<Self> {
        let pattern = "4c 8b dc 55 53 56 41 54 41 56 49 8d 6b d1";

        let ptr = main_module_memory.find_pattern(pattern).unwrap()?;

        let f = unsafe { transmute::<*const u8, PlaceBlockFnType>(ptr) };

        Some(Self(f))
    }

    pub unsafe fn call(&self, this: &mut EditorCommon, block_info: &mut BlockInfo) -> usize {
        let mut coord = [20, 20, 20];

        (self.0)(
            this,
            block_info,
            coord.as_mut_ptr(),
            0,
            0,
            0,
            1,
            0,
            0xffffffff,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            1,
        )
    }
}
