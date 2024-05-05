mod fns;
mod hook;

mod engines {
    mod game;
    mod game_data;
    mod mw_foundations;
    mod system;

    pub use game::*;
    pub use game_data::*;
    pub use mw_foundations::*;
    pub use system::*;
}

pub use engines::*;
pub use fns::*;
pub use hook::*;

use autopad::autopad;

use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
    str,
};

pub trait Class {
    const ID: u32;
}

pub fn cast_nod<T: Class>(nod: &Nod) -> Option<&T> {
    if nod.is_instance_of(T::ID) {
        unsafe { Some(&*(nod as *const _ as *const _)) }
    } else {
        None
    }
}

autopad! {
    /// CGameManiaPlanet.
    #[repr(C)]
    pub struct ManiaPlanet {
        0x7f0 => switcher: *mut Switcher,
    }
}

impl ManiaPlanet {
    pub fn switcher(&self) -> &Switcher {
        unsafe { &*self.switcher }
    }
}

autopad! {
    /// CGameSwitcher.
    #[repr(C)]
    pub struct Switcher {
        0x20 => modules: Array<Nod>
    }
}

impl Switcher {
    pub fn modules(&self) -> &[&Nod] {
        self.modules.as_slice()
    }
}

/// Reference-counted pointer to a [Nod].
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
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl<T: DerefMut<Target = Nod>> Drop for NodRef<T> {
    fn drop(&mut self) {
        let nod = unsafe { (*self.ptr).deref_mut() };

        nod.ref_count -= 1;

        if nod.ref_count == 0 {
            unsafe { ((*(nod.vtable)).destructor)(nod, true) };
        }
    }
}

impl<T: DerefMut<Target = Nod>> From<&T> for NodRef<T> {
    fn from(x: &T) -> Self {
        let ptr = x as *const T as *mut T;
        let nod = unsafe { (*ptr).deref_mut() };

        nod.ref_count += 1;

        Self { ptr }
    }
}

pub fn fids_folder_full_path(folder: &FidsFolder) -> PathBuf {
    if let Some(parent_folder) = folder.parent_folder() {
        let mut path = fids_folder_full_path(parent_folder);
        path.push(folder.name());

        path
    } else {
        PathBuf::from(folder.name())
    }
}

pub fn fids_folder_get_subfolder<'a>(folder: &'a FidsFolder, name: &str) -> Option<&'a FidsFolder> {
    folder
        .trees()
        .iter()
        .find(|folder| folder.name() == name)
        .copied()
}
