use std::{mem::MaybeUninit, slice, str};

use autopad::autopad;

use super::{Article, FidFile};

autopad! {
    // CMwNod.
    #[repr(C)]
    pub struct Nod {
                pub vtable: *const NodVtable,
        0x08 =>     file: *const FidFile,
        0x10 => pub ref_count: u32,
        0x18 =>     article: *mut Article,
    }
}

autopad! {
    #[repr(C)]
    pub struct NodVtable {
        0x08 => pub destructor: unsafe extern "system" fn(this: *mut Nod, should_free: bool) -> *mut Nod,
        0x18 =>     class_id: unsafe extern "system" fn(this: *const Nod, class_id: *mut u32) -> *mut u32,
        0x20 =>     is_instance_of: unsafe extern "system" fn(this: *const Nod, class_id: u32) -> bool,
    }
}

impl Nod {
    pub fn file(&self) -> &FidFile {
        unsafe { &*self.file }
    }

    pub fn article(&self) -> &Article {
        unsafe { &*self.article }
    }

    pub fn class_id(&self) -> u32 {
        let mut class_id: MaybeUninit<u32> = MaybeUninit::uninit();

        unsafe { ((*self.vtable).class_id)(self, class_id.as_mut_ptr()) };

        unsafe { class_id.assume_init() }
    }

    pub fn is_instance_of(&self, class_id: u32) -> bool {
        unsafe { ((*self.vtable).is_instance_of)(self, class_id) }
    }
}

#[repr(C)]
pub struct Array<T> {
    ptr: *const *const T,
    len: u32,
    cap: u32,
}

impl<T> Array<T> {
    pub fn as_slice(&self) -> &[&T] {
        unsafe { slice::from_raw_parts(self.ptr as _, self.len as usize) }
    }
}

#[repr(C)]
pub struct CompactString {
    data: [u8; 12],
    len: u32,
}

impl CompactString {
    pub fn as_str(&self) -> &str {
        let bytes = if self.len as usize >= self.data.len() || self.data[self.data.len() - 1] != 0 {
            let ptr = usize::from_le_bytes(self.data[..8].try_into().unwrap()) as *const u8;
            unsafe { slice::from_raw_parts(ptr, self.len as usize) }
        } else {
            &self.data[..self.len as usize]
        };

        str::from_utf8(bytes).expect("string is not valid UTF-8")
    }
}
