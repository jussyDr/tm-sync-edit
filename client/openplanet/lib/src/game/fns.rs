use std::mem::transmute;

use crate::process::ModuleMemory;

use super::{ManiaPlanet, ManiaTitleControlScriptApi};

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

    pub unsafe fn call(&self, this: &mut ManiaTitleControlScriptApi) {
        let mut arg_1: [u8; 16] = [
            0x53, 0x74, 0x61, 0x64, 0x69, 0x75, 0x6d, 0, 0, 0, 0, 0, 7, 0, 0, 0,
        ];

        let mut p2_str: [u8; 22] = [
            0x34, 0x38, 0x78, 0x34, 0x38, 0x53, 0x63, 0x72, 0x65, 0x65, 0x6e, 0x31, 0x35, 0x35,
            0x44, 0x61, 0x79, 0, 0, 0, 0, 0,
        ];

        let mut arg_2: Vec<u8> = vec![];
        arg_2.extend((p2_str.as_mut_ptr() as usize).to_le_bytes());
        arg_2.extend([0, 0, 0, 1, 0x11, 0, 0, 0]);

        let mut arg_3: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let mut arg_4: [u8; 16] = [
            0x43, 0x61, 0x72, 0x53, 0x70, 0x6f, 0x72, 0x74, 0, 0, 0, 0, 8, 0, 0, 0,
        ];

        let mut arg_5: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let mut arg_6: [u8; 4] = [0, 0, 0, 0];

        let mut arg_7: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let mut arg_8: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let mut extra_args: Vec<u8> = vec![];
        extra_args.extend((arg_5.as_mut_ptr() as usize).to_le_bytes());
        extra_args.extend([0x7e, 0, 0, 0, 0, 0, 0, 0]);
        extra_args.extend((arg_4.as_mut_ptr() as usize).to_le_bytes());
        extra_args.extend([0x7e, 0, 0, 0, 0, 0, 0, 0]);
        extra_args.extend((arg_3.as_mut_ptr() as usize).to_le_bytes());
        extra_args.extend([0x7e, 0, 0, 0, 0, 0, 0, 0]);
        extra_args.extend((arg_2.as_mut_ptr() as usize).to_le_bytes());
        extra_args.extend([0x7d, 0, 0, 0, 0, 0, 0, 0]);
        extra_args.extend((arg_1.as_mut_ptr() as usize).to_le_bytes());
        extra_args.extend([0x7d, 0, 0, 0, 0, 0, 0, 0]);

        let mut args = vec![];
        args.extend(&[1, 9, 0, 0, 0x7d, 0x7e, 0x71, 0]);
        args.extend(0usize.to_le_bytes());
        args.extend((arg_8.as_mut_ptr() as usize).to_le_bytes());
        args.extend((arg_7.as_mut_ptr() as usize).to_le_bytes());
        args.extend((arg_6.as_mut_ptr() as usize).to_le_bytes());
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
