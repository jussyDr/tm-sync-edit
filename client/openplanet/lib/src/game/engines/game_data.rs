use std::ops::{Deref, DerefMut};

use autopad::autopad;

use crate::game::Class;

use super::Nod;

autopad! {
    // CGameItemModel.
    #[repr(C)]
    pub struct ItemModel {
                     nod: Nod,
        0x028 => pub id: u32,
        0x288 =>     entity_model: *const Nod,
    }
}

impl ItemModel {
    pub fn entity_model(&self) -> Option<&Nod> {
        if self.entity_model.is_null() {
            None
        } else {
            unsafe { Some(&*self.entity_model) }
        }
    }
}

impl Class for ItemModel {
    const ID: u32 = 0x2e002000;
}

impl Deref for ItemModel {
    type Target = Nod;

    fn deref(&self) -> &Nod {
        &self.nod
    }
}

impl DerefMut for ItemModel {
    fn deref_mut(&mut self) -> &mut Nod {
        &mut self.nod
    }
}
