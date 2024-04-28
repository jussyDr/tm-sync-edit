use std::{
    error::Error,
    ffi::{c_char, CStr},
    mem::{transmute, MaybeUninit},
    ptr::null_mut,
};

use memchr::memmem;
use ordered_float::NotNan;
use shared::{Direction, ElemColor};

use super::{Block, BlockInfo, FidFile, Item, ItemModel, ItemParams, MapEditor, Nod};

#[derive(Clone, Copy)]
pub struct PreloadFidFn(
    unsafe extern "system" fn(ret_nod: *mut *mut Nod, fid: *mut FidFile, nod: *mut u8),
);

impl PreloadFidFn {
    pub fn find(exe_module_memory: &[u8]) -> Result<Self, Box<dyn Error>> {
        let preload_fid_fn_offset = memmem::find(
            exe_module_memory,
            &[
                0x48, 0x33, 0xc4, 0x48, 0x89, 0x84, 0x24, 0xa0, 0x00, 0x00, 0x00, 0x49, 0x8b, 0xf8,
            ],
        )
        .ok_or("failed to find PreloadFid function pattern")?
            - 18;

        let preload_fid_fn =
            unsafe { transmute(exe_module_memory.as_ptr().add(preload_fid_fn_offset)) };

        Ok(Self(preload_fid_fn))
    }

    pub unsafe fn call(&self, fid: *mut FidFile) -> Option<&Nod> {
        let mut ret_nod = MaybeUninit::uninit();
        let mut nod = [0; 32];

        unsafe { (self.0)(ret_nod.as_mut_ptr(), fid, nod.as_mut_ptr()) };

        let ret_nod = unsafe { ret_nod.assume_init() };

        if ret_nod.is_null() {
            None
        } else {
            Some(&*ret_nod)
        }
    }
}

#[derive(Clone, Copy)]
pub struct IdNameFn(unsafe extern "system" fn(id: *const u32) -> *mut c_char);

impl IdNameFn {
    pub fn find(exe_module_memory: &[u8]) -> Result<Self, Box<dyn Error>> {
        let id_name_fn_offset = memmem::find(exe_module_memory, &[0x8b, 0x11, 0x8b, 0xc2, 0x25])
            .ok_or("failed to find IdName function pattern")?;

        let id_name_fn = unsafe { transmute(exe_module_memory.as_ptr().add(id_name_fn_offset)) };

        Ok(Self(id_name_fn))
    }

    pub fn call(&self, id: u32) -> String {
        let id_name = unsafe { (self.0)(&id) };

        unsafe { CStr::from_ptr(id_name).to_str().unwrap().to_owned() }
    }
}

pub struct PlaceBlockFn(
    unsafe extern "system" fn(
        map_editor: *mut MapEditor,
        block_info: *const BlockInfo,
        param_3: usize,
        coord: *mut [u32; 3],
        dir: u32,
        elem_color: u8,
        param_7: u8,
        param_8: u32,
        param_9: u32,
        is_ghost: u32,
        place_pillars: u32,
        param_12: u32,
        is_ground: u32,
        param_14: u32,
        is_ghost: u32,
        param_16: usize,
        is_free: u32,
        transform: *mut [f32; 6],
        param_19: u32,
        param_20: u32,
    ) -> *mut Block,
);

impl PlaceBlockFn {
    pub fn find(exe_module_memory: &[u8]) -> Result<Self, Box<dyn Error>> {
        let place_block_fn_offset = memmem::find(
            exe_module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x20, 0x4c, 0x89, 0x44, 0x24,
                0x18, 0x55,
            ],
        )
        .ok_or("failed to find PlaceBlock function pattern")?;

        let place_block_fn =
            unsafe { transmute(exe_module_memory.as_ptr().add(place_block_fn_offset)) };

        Ok(Self(place_block_fn))
    }

    #[allow(clippy::too_many_arguments)]
    pub unsafe fn call_normal(
        &self,
        map_editor: &mut MapEditor,
        block_info: &BlockInfo,
        x: u8,
        y: u8,
        z: u8,
        dir: Direction,
        elem_color: ElemColor,
        is_ghost: bool,
        is_ground: bool,
    ) -> Option<&Block> {
        let mut coord = [x as u32, y as u32, z as u32];

        let block = (self.0)(
            map_editor,
            block_info,
            0,
            &mut coord,
            dir as u32,
            elem_color as u8,
            0,
            0,
            0xffffffff,
            is_ghost as u32,
            0,
            0,
            is_ground as u32,
            0,
            is_ghost as u32,
            0,
            0,
            null_mut(),
            0xffffffff,
            0,
        );

        if block.is_null() {
            None
        } else {
            Some(&*block)
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub unsafe fn call_free(
        &self,
        map_editor: &mut MapEditor,
        block_info: &mut BlockInfo,
        elem_color: ElemColor,
        x: f32,
        y: f32,
        z: f32,
        yaw: f32,
        pitch: f32,
        roll: f32,
    ) -> Option<&Block> {
        let mut coord = [0xffffffff, 0, 0xffffffff];

        let mut transform = [x, y, z, yaw, pitch, roll];

        let block = (self.0)(
            map_editor,
            block_info,
            0,
            &mut coord,
            0,
            elem_color as u8,
            0,
            0,
            0x3f,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            1,
            &mut transform,
            0xffffffff,
            0,
        );

        if block.is_null() {
            None
        } else {
            Some(&*block)
        }
    }
}

pub struct RemoveBlockFn(
    unsafe extern "system" fn(
        map_editor: *mut MapEditor,
        block: *mut Block,
        param_3: u32,
        param_4: *mut Block,
        param_5: u32,
    ) -> u32,
);

impl RemoveBlockFn {
    pub fn find(exe_module_memory: &[u8]) -> Result<Self, Box<dyn Error>> {
        let remove_block_fn_offset = memmem::find(
            exe_module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x6c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24,
                0x18, 0x57, 0x48, 0x83, 0xec, 0x40, 0x83, 0x7c, 0x24, 0x70, 0x00,
            ],
        )
        .ok_or("failed to find RemoveBlock function pattern")?;

        let remove_block_fn =
            unsafe { transmute(exe_module_memory.as_ptr().add(remove_block_fn_offset)) };

        Ok(Self(remove_block_fn))
    }

    pub unsafe fn call(&self, map_editor: &mut MapEditor, block: &mut Block) -> u32 {
        (self.0)(map_editor, block, 1, block, 0)
    }
}

pub struct PlaceItemFn(
    unsafe extern "system" fn(
        map_editor: *mut MapEditor,
        item_model: *const ItemModel,
        params: *mut ItemParams,
        out_item: *mut *mut Item,
    ) -> u32,
);

impl PlaceItemFn {
    pub fn find(exe_module_memory: &[u8]) -> Result<Self, Box<dyn Error>> {
        let place_item_fn_offset = memmem::find(
            exe_module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x6c, 0x24, 0x18, 0x48, 0x89, 0x74, 0x24,
                0x20, 0x57, 0x48, 0x83, 0xec, 0x40, 0x49, 0x8b, 0xf9,
            ],
        )
        .ok_or("failed to find PlaceItem function pattern")?;

        let place_item_fn =
            unsafe { transmute(exe_module_memory.as_ptr().add(place_item_fn_offset)) };

        Ok(Self(place_item_fn))
    }

    #[allow(clippy::too_many_arguments)]
    pub unsafe fn call(
        &self,
        map_editor: &mut MapEditor,
        item_model: &ItemModel,
        yaw: NotNan<f32>,
        pitch: NotNan<f32>,
        roll: NotNan<f32>,
        x: NotNan<f32>,
        y: NotNan<f32>,
        z: NotNan<f32>,
    ) -> u32 {
        let coord = coord_from_pos([x.into_inner(), y.into_inner(), z.into_inner()]);

        let mut params = ItemParams {
            x_coord: coord[0],
            y_coord: coord[1],
            z_coord: coord[2],
            yaw,
            pitch,
            roll,
            param_7: 0xffffffff,
            x_pos: x,
            y_pos: y,
            z_pos: z,
            param_11: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            param_12: [0.0, 0.0, 0.0],
            param_13: 1.0,
            param_14: 1,
            param_15: 0xffffffff,
            param_16: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            param_17: [-1.0, -1.0, -1.0],
            param_18: 0xfe000000,
            param_19: 0,
        };

        unsafe { (self.0)(map_editor, item_model, &mut params, null_mut()) }
    }
}

pub struct RemoveItemFn(
    unsafe extern "system" fn(map_editor: *mut MapEditor, item: *mut Item) -> u32,
);

impl RemoveItemFn {
    pub fn find(exe_module_memory: &[u8]) -> Result<Self, Box<dyn Error>> {
        let remove_item_fn_offset = memmem::find(
            exe_module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x08, 0x57, 0x48, 0x83, 0xec, 0x30, 0x48, 0x8b, 0xfa, 0x48,
                0x8b, 0xd9, 0x48, 0x85, 0xd2, 0x0f, 0x84, 0xe6, 0x00, 0x00, 0x00,
            ],
        )
        .ok_or("failed to find RemoveItem function pattern")?;

        let remove_item_fn =
            unsafe { transmute(exe_module_memory.as_ptr().add(remove_item_fn_offset)) };

        Ok(Self(remove_item_fn))
    }

    pub unsafe fn call(&self, map_editor: &mut MapEditor, item: &mut Item) -> u32 {
        (self.0)(map_editor, item)
    }
}

fn coord_from_pos(pos: [f32; 3]) -> [u32; 3] {
    [
        (pos[0] as u32) / 32,
        ((pos[1] + 64.0) as u32) / 8,
        (pos[2] as u32) / 32,
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn coord_from_pos() {
        for (pos, expected_coord) in [
            ([0.0, -56.0, 0.0], [0, 1, 0]),
            ([31.0, -49.0, 31.0], [0, 1, 0]),
            ([32.0, -48.0, 32.0], [1, 2, 1]),
            ([1503.0, 247.0, 1503.0], [46, 38, 46]),
            ([1504.0, 248.0, 1504.0], [47, 39, 47]),
            ([1535.0, 255.0, 1535.0], [47, 39, 47]),
        ] {
            let actual_coord = super::coord_from_pos(pos);
            assert_eq!(actual_coord, expected_coord);
        }
    }
}
