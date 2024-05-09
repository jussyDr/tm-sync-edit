use std::{
    error::Error,
    ffi::{c_char, CStr},
    mem::{transmute_copy, MaybeUninit},
    ptr::{null, null_mut},
};

use gamebox::{
    engines::game::map::{Direction, ElemColor, PhaseOffset},
    Vec3,
};
use memchr::memmem;
use ordered_float::NotNan;

use super::{Block, BlockInfo, FidFile, Item, ItemModel, ItemParams, MapEditor, Nod};

fn find_fn<T>(
    exe_module_memory: &[u8],
    pattern: &[u8],
    pattern_offset: isize,
) -> Result<T, Box<dyn Error>> {
    let offset = memmem::find(exe_module_memory, pattern)
        .ok_or("failed to find function pattern")? as isize
        + pattern_offset;

    let f = unsafe { transmute_copy::<*const u8, T>(&exe_module_memory.as_ptr().offset(offset)) };

    Ok(f)
}

type PreloadFidFnType =
    unsafe extern "system" fn(ret_nod: *mut *mut Nod, fid: *mut FidFile, nod: *mut u8);

#[derive(Clone, Copy)]
pub struct PreloadFidFn(PreloadFidFnType);

impl PreloadFidFn {
    pub fn find(exe_module_memory: &[u8]) -> Result<Self, Box<dyn Error>> {
        let pattern = &[
            0x48, 0x33, 0xc4, 0x48, 0x89, 0x84, 0x24, 0xa0, 0x00, 0x00, 0x00, 0x49, 0x8b, 0xf8,
        ];

        let preload_fid_fn = find_fn(exe_module_memory, pattern, -18)?;

        Ok(Self(preload_fid_fn))
    }

    pub unsafe fn call(&self, fid: *mut FidFile) -> Option<&mut Nod> {
        let mut ret_nod = MaybeUninit::uninit();
        let mut nod = [0; 32];

        unsafe { (self.0)(ret_nod.as_mut_ptr(), fid, nod.as_mut_ptr()) };

        let ret_nod = unsafe { ret_nod.assume_init() };

        if ret_nod.is_null() {
            None
        } else {
            Some(&mut *ret_nod)
        }
    }
}

type IdNameFnType = unsafe extern "system" fn(id: *const u32) -> *mut c_char;

#[derive(Clone, Copy)]
pub struct IdNameFn(IdNameFnType);

impl IdNameFn {
    pub fn find(exe_module_memory: &[u8]) -> Result<Self, Box<dyn Error>> {
        let pattern = &[0x8b, 0x11, 0x8b, 0xc2, 0x25];

        let id_name_fn = find_fn(exe_module_memory, pattern, 0)?;

        Ok(Self(id_name_fn))
    }

    pub fn call(&self, id: u32) -> String {
        let id_name = unsafe { (self.0)(&id) };

        unsafe { CStr::from_ptr(id_name).to_str().unwrap().to_owned() }
    }
}

type PreloadBlockInfoFnType =
    unsafe extern "system" fn(item_model: *mut ItemModel, file: *const FidFile, param_3: u32);

pub struct PreloadBlockInfoFn(PreloadBlockInfoFnType);

impl PreloadBlockInfoFn {
    pub fn find(exe_module_memory: &[u8]) -> Result<Self, Box<dyn Error>> {
        let pattern = &[
            0x40, 0x55, 0x56, 0x41, 0x54, 0x41, 0x56, 0x48, 0x8d, 0xac, 0x24, 0x98, 0xfa, 0xff,
            0xff,
        ];

        let preload_block_info_fn = find_fn(exe_module_memory, pattern, 0)?;

        Ok(Self(preload_block_info_fn))
    }

    pub fn call(&self, item_model: &mut ItemModel, file: &FidFile) {
        unsafe { (self.0)(item_model, file, 0xffffffff) }
    }
}

type PlaceBlockFnType = unsafe extern "system" fn(
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
) -> *mut Block;

pub struct PlaceBlockFn(PlaceBlockFnType);

impl PlaceBlockFn {
    pub fn find(exe_module_memory: &[u8]) -> Result<Self, Box<dyn Error>> {
        let pattern = &[
            0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x20, 0x4c, 0x89, 0x44, 0x24,
            0x18, 0x55,
        ];

        let place_block_fn = find_fn(exe_module_memory, pattern, 0)?;

        Ok(Self(place_block_fn))
    }

    #[allow(clippy::too_many_arguments)]
    pub unsafe fn call_normal(
        &self,
        map_editor: &mut MapEditor,
        block_info: &BlockInfo,
        coordinate: Vec3<u8>,
        dir: Direction,
        elem_color: ElemColor,
        is_ghost: bool,
        is_ground: bool,
    ) -> Option<&Block> {
        let mut coord = [
            coordinate.x as u32,
            coordinate.y as u32,
            coordinate.z as u32,
        ];

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
        block_info: &BlockInfo,
        elem_color: ElemColor,
        position: Vec3<NotNan<f32>>,
        yaw: NotNan<f32>,
        pitch: NotNan<f32>,
        roll: NotNan<f32>,
    ) -> Option<&Block> {
        let mut coord = [0xffffffff, 0, 0xffffffff];

        let mut transform = [
            position.x.into_inner(),
            position.y.into_inner(),
            position.z.into_inner(),
            yaw.into_inner(),
            pitch.into_inner(),
            roll.into_inner(),
        ];

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

type RemoveBlockFnType = unsafe extern "system" fn(
    map_editor: *mut MapEditor,
    block: *mut Block,
    param_3: u32,
    param_4: *mut Block,
    param_5: u32,
) -> u32;

pub struct RemoveBlockFn(RemoveBlockFnType);

impl RemoveBlockFn {
    pub fn find(exe_module_memory: &[u8]) -> Result<Self, Box<dyn Error>> {
        let pattern = &[
            0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x6c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24,
            0x18, 0x57, 0x48, 0x83, 0xec, 0x40, 0x83, 0x7c, 0x24, 0x70, 0x00,
        ];

        let place_block_fn = find_fn(exe_module_memory, pattern, 0)?;

        Ok(Self(place_block_fn))
    }

    pub unsafe fn call(&self, map_editor: &mut MapEditor, block: &mut Block) -> u32 {
        (self.0)(map_editor, block, 1, block, 0)
    }
}

type PlaceItemFnType = unsafe extern "system" fn(
    map_editor: *mut MapEditor,
    item_model: *const ItemModel,
    params: *mut ItemParams,
    out_item: *mut *mut Item,
) -> u32;

pub struct PlaceItemFn(PlaceItemFnType);

impl PlaceItemFn {
    pub fn find(exe_module_memory: &[u8]) -> Result<Self, Box<dyn Error>> {
        let pattern = &[
            0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x6c, 0x24, 0x18, 0x48, 0x89, 0x74, 0x24,
            0x20, 0x57, 0x48, 0x83, 0xec, 0x40, 0x49, 0x8b, 0xf9,
        ];

        let place_block_fn = find_fn(exe_module_memory, pattern, 0)?;

        Ok(Self(place_block_fn))
    }

    #[allow(clippy::too_many_arguments)]
    pub unsafe fn call(
        &self,
        map_editor: &mut MapEditor,
        item_model: &ItemModel,
        yaw: NotNan<f32>,
        pitch: NotNan<f32>,
        roll: NotNan<f32>,
        position: Vec3<NotNan<f32>>,
        pivot_position: Vec3<NotNan<f32>>,
        elem_color: ElemColor,
        anim_offset: PhaseOffset,
    ) -> u32 {
        let coordinate = coordinate_from_position(&position);

        let mut params = ItemParams {
            coordinate,
            yaw,
            pitch,
            roll,
            param_5: 0xffffffff,
            position,
            param_7: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            pivot_position,
            param_9: 1.0,
            param_10: 0,
            param_11: 0xffffffff,
            param_12: 0,
            parent_block: null(),
            skin: null(),
            skin_effect: null(),
            param_16: [0, 0, 0],
            param_17: [-1.0, -1.0, -1.0],
            elem_color,
            anim_offset,
        };

        unsafe { (self.0)(map_editor, item_model, &mut params, null_mut()) }
    }
}

type RemoveItemFnType =
    unsafe extern "system" fn(map_editor: *mut MapEditor, item: *mut Item) -> u32;

pub struct RemoveItemFn(RemoveItemFnType);

impl RemoveItemFn {
    pub fn find(exe_module_memory: &[u8]) -> Result<Self, Box<dyn Error>> {
        let pattern = &[
            0x48, 0x89, 0x5c, 0x24, 0x08, 0x57, 0x48, 0x83, 0xec, 0x30, 0x48, 0x8b, 0xfa, 0x48,
            0x8b, 0xd9, 0x48, 0x85, 0xd2, 0x0f, 0x84, 0xe6, 0x00, 0x00, 0x00,
        ];

        let remove_item_fn = find_fn(exe_module_memory, pattern, 0)?;

        Ok(Self(remove_item_fn))
    }

    pub unsafe fn call(&self, map_editor: &mut MapEditor, item: &mut Item) -> u32 {
        (self.0)(map_editor, item)
    }
}

fn coordinate_from_position(position: &Vec3<NotNan<f32>>) -> Vec3<u32> {
    Vec3 {
        x: (position.x.into_inner() as u32) / 32,
        y: ((position.y.into_inner() + 64.0) as u32) / 8,
        z: (position.z.into_inner() as u32) / 32,
    }
}

#[cfg(test)]
mod tests {
    use gamebox::Vec3;
    use ordered_float::NotNan;

    #[test]
    fn coordinate_from_position() {
        for (position, expected_coord) in [
            (
                Vec3 {
                    x: NotNan::new(0.0).unwrap(),
                    y: NotNan::new(-56.0).unwrap(),
                    z: NotNan::new(0.0).unwrap(),
                },
                Vec3 { x: 0, y: 1, z: 0 },
            ),
            (
                Vec3 {
                    x: NotNan::new(31.0).unwrap(),
                    y: NotNan::new(-49.0).unwrap(),
                    z: NotNan::new(31.0).unwrap(),
                },
                Vec3 { x: 0, y: 1, z: 0 },
            ),
            (
                Vec3 {
                    x: NotNan::new(32.0).unwrap(),
                    y: NotNan::new(-48.0).unwrap(),
                    z: NotNan::new(32.0).unwrap(),
                },
                Vec3 { x: 1, y: 2, z: 1 },
            ),
            (
                Vec3 {
                    x: NotNan::new(1503.0).unwrap(),
                    y: NotNan::new(247.0).unwrap(),
                    z: NotNan::new(1503.0).unwrap(),
                },
                Vec3 {
                    x: 46,
                    y: 38,
                    z: 46,
                },
            ),
            (
                Vec3 {
                    x: NotNan::new(1504.0).unwrap(),
                    y: NotNan::new(248.0).unwrap(),
                    z: NotNan::new(1504.0).unwrap(),
                },
                Vec3 {
                    x: 47,
                    y: 39,
                    z: 47,
                },
            ),
            (
                Vec3 {
                    x: NotNan::new(1535.0).unwrap(),
                    y: NotNan::new(255.0).unwrap(),
                    z: NotNan::new(1535.0).unwrap(),
                },
                Vec3 {
                    x: 47,
                    y: 39,
                    z: 47,
                },
            ),
        ] {
            let actual_coord = super::coordinate_from_position(&position);
            assert_eq!(actual_coord, expected_coord);
        }
    }
}
