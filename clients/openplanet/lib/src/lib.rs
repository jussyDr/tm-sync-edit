use std::ffi::c_void;
use std::ptr::null;

#[derive(PartialEq, Debug)]
#[repr(C)]
struct Vec3<T>([T; 3]);

#[repr(C)]
struct Transform {
    pos: Vec3<f32>,
    rot: Vec3<f32>,
}

type PlaceBlockFn = unsafe extern "win64" fn(
    editor: *const c_void,
    block_info: *const c_void,
    *const c_void,
    coord: *const Vec3<i32>,
    dir: u32,
    color: u8,
    u8,
    u32,
    i32,
    is_ghost: u32,
    place_pillars: u32,
    u32,
    is_ground: u32,
    u32,
    u32,
    *const c_void,
    is_free: u32,
    transform: *const Transform,
    i32,
    i32,
) -> *const c_void;

#[no_mangle]
unsafe extern "win64" fn PlaceBlock(
    place_block_fn: PlaceBlockFn,
    editor: *const c_void,
    block_info: *const c_void,
    coord: Vec3<i32>,
    dir: u32,
    is_ground: u32,
    is_ghost: u32,
    color: u8,
) -> *const c_void {
    place_block_fn(
        editor,
        block_info,
        null(),
        &coord,
        dir,
        color,
        0,
        0,
        -1,
        is_ghost,
        0,
        0,
        is_ground,
        0,
        0,
        null(),
        0,
        null(),
        -1,
        0,
    )
}

#[no_mangle]
unsafe extern "win64" fn PlaceFreeBlock(
    place_block_fn: PlaceBlockFn,
    editor: *const c_void,
    block_info: *const c_void,
    pos: Vec3<f32>,
    rot: Vec3<f32>,
    color: u8,
) -> *const c_void {
    let coord = Vec3([-1, 0, -1]);
    let transform = Transform { pos, rot };

    place_block_fn(
        editor,
        block_info,
        null(),
        &coord,
        0,
        color,
        0,
        0,
        63,
        0,
        0,
        0,
        0,
        0,
        0,
        null(),
        1,
        &transform,
        -1,
        0,
    )
}

type RemoveBlockFn = unsafe extern "win64" fn(
    editor: *const c_void,
    block: *const c_void,
    u32,
    *const c_void,
    u32,
) -> *const c_void;

#[no_mangle]
unsafe extern "win64" fn RemoveBlock(
    remove_block_fn: RemoveBlockFn,
    editor: *const c_void,
    block: *const c_void,
) {
    remove_block_fn(editor, block, 0, null(), 0);
}

type PlaceItemFn = unsafe extern "win64" fn(
    editor: *const c_void,
    item_model: *const c_void,
    params: *const ItemParams,
    item: *mut *const c_void,
) -> u32;

#[repr(C)]
struct ItemParams {
    coord: Vec3<u32>,
    rot: Vec3<f32>,
    unknown_1: i32,
    pos: Vec3<f32>,
    unknown_2: [f32; 9],
    pivot_pos: Vec3<f32>,
    unknown_3: f32,
    is_free: u32,
    unknown_4: u32,
    unknown_5: i32,
    unknown_6: [u32; 9],
    unknown_7: Vec3<f32>,
    color: u8,
    anim_offset: u8,
    unknown_8: u8,
    unknown_9: u8,
    unknown_10: i32,
}

fn coord_from_pos(Vec3([x, y, z]): &Vec3<f32>) -> Vec3<u32> {
    Vec3([(x / 32.0) as u32, (y / 8.0 + 8.0) as u32, (z / 32.0) as u32])
}

#[no_mangle]
unsafe extern "win64" fn PlaceItem(
    place_item_fn: PlaceItemFn,
    editor: *const c_void,
    item_model: *const c_void,
    pos: Vec3<f32>,
    rot: Vec3<f32>,
    pivot_pos: Vec3<f32>,
    color: u8,
    anim_offset: u8,
) -> *const c_void {
    let params = ItemParams {
        coord: coord_from_pos(&pos),
        rot,
        unknown_1: -1,
        pos,
        unknown_2: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        pivot_pos,
        unknown_3: 1.0,
        is_free: 1,
        unknown_4: 0x11150000,
        unknown_5: -1,
        unknown_6: [0, 0, 0, 0, 0, 0, 0, 0, 0],
        unknown_7: Vec3([-1.0, -1.0, -1.0]),
        color,
        anim_offset,
        unknown_8: 0,
        unknown_9: 68,
        unknown_10: -1,
    };

    let item = &mut null();

    place_item_fn(editor, item_model, &params, item);

    *item
}

type RemoveItemFn =
    unsafe extern "win64" fn(editor: *const c_void, item: *const c_void, i32, *const c_void) -> u32;

#[no_mangle]
unsafe extern "win64" fn RemoveItem(
    remove_item_fn: RemoveItemFn,
    editor: *const c_void,
    item: *const c_void,
) {
    remove_item_fn(editor, item, -8, null());
}

type LoadBlockInfoFn = unsafe extern "win64" fn(block_item: *const c_void) -> *const c_void;

#[no_mangle]
unsafe extern "win64" fn LoadBlockInfo(
    load_block_info_fn: LoadBlockInfoFn,
    block_item: *const c_void,
) -> *const c_void {
    load_block_info_fn(block_item)
}

#[cfg(test)]
mod tests {
    use super::{coord_from_pos, Vec3};

    #[test]
    fn test_coord_from_pos() {
        for (pos, expected_coord) in [
            (Vec3([31.0, -49.0, 31.0]), Vec3([0, 1, 0])),
            (Vec3([32.0, -48.0, 32.0]), Vec3([1, 2, 1])),
            (Vec3([1503.0, 247.0, 1503.0]), Vec3([46, 38, 46])),
            (Vec3([1504.0, 248.0, 1504.0]), Vec3([47, 39, 47])),
        ] {
            println!("{pos:?}");

            let coord = coord_from_pos(&pos);

            assert_eq!(coord, expected_coord);
        }
    }
}
