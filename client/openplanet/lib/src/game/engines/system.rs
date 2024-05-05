use std::{ffi::c_char, slice, str};

use autopad::autopad;

use super::{Array, CompactString, Nod};

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

#[repr(C)]
pub struct PackDesc;
