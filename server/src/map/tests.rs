use super::{Block, Map, ModelRef};
use gbx::map::{Color, Direction};
use gbx::Vec3;
use std::borrow::Cow;

fn can_place_block(map: &Map, block: &Block) -> bool {
    let block_info = if let Some(block_info) = map.get_block_info(&block.model) {
        block_info
    } else {
        return false;
    };

    let variant = if let Some(variant) = block_info.variant(block.is_ground, block.variant_index) {
        variant
    } else {
        return false;
    };

    map.can_place_block(block, variant)
}

#[test]
fn can_place_block_unit_intersection() {
    let mut map = Map::new();

    let coord = Vec3::new(20, 20, 20);

    map.place_block(Block {
        model: ModelRef::Id(Cow::Borrowed("PlatformBase")),
        coord,
        dir: Direction::North,
        is_ground: false,
        is_ghost: false,
        variant_index: 0,
        color: Color::Default,
    });

    for (coord, no_intersection_dir) in [
        (Vec3::new(coord.x, coord.y, coord.z - 2), Direction::North),
        (coord, Direction::East),
        (Vec3::new(coord.x - 2, coord.y, coord.z), Direction::South),
        (
            Vec3::new(coord.x - 2, coord.y, coord.z - 2),
            Direction::West,
        ),
    ] {
        for dir in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            println!("{coord:?} {dir:?}");

            let can_place = can_place_block(
                &map,
                &Block {
                    model: ModelRef::Id(Cow::Borrowed("TrackWallCurve3")),
                    coord,
                    dir,
                    is_ground: false,
                    variant_index: 0,
                    is_ghost: false,
                    color: Color::Default,
                },
            );

            assert_eq!(can_place, dir == no_intersection_dir);
        }
    }
}

#[test]
fn can_place_block_clipping() {
    let mut map = Map::new();

    let coord = Vec3::new(20, 20, 20);

    map.place_block(Block {
        model: ModelRef::Id(Cow::Borrowed("PlatformBase")),
        coord,
        dir: Direction::North,
        is_ground: false,
        is_ghost: false,
        variant_index: 0,
        color: Color::Default,
    });

    for (coord, no_clip_dir) in [
        (Vec3::new(coord.x - 1, coord.y, coord.z), Direction::North),
        (Vec3::new(coord.x, coord.y, coord.z - 1), Direction::East),
        (Vec3::new(coord.x + 1, coord.y, coord.z), Direction::South),
        (Vec3::new(coord.x, coord.y, coord.z + 1), Direction::West),
    ] {
        for dir in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            println!("{coord:?} {dir:?}");

            let can_place = can_place_block(
                &map,
                &Block {
                    model: ModelRef::Id(Cow::Borrowed("RoadTechBranchTShaped")),
                    coord,
                    dir,
                    is_ground: false,
                    variant_index: 0,
                    is_ghost: false,
                    color: Color::Default,
                },
            );

            assert_eq!(can_place, dir == no_clip_dir);
        }
    }
}

#[test]
fn place_out_of_bounds() {
    let mut map = Map::new();

    for coord in [
        Vec3::new(48, 0, 0),
        Vec3::new(0, 40, 0),
        Vec3::new(0, 0, 48),
    ] {
        println!("{coord:?}");

        let block = Block {
            model: ModelRef::Id(Cow::Borrowed("PlatformBase")),
            coord,
            dir: Direction::North,
            is_ground: false,
            is_ghost: false,
            variant_index: 0,
            color: Color::Default,
        };

        assert!(!map.place_block(block))
    }
}

#[test]
fn remove_place_block() {
    let mut map = Map::new();

    let block = Block {
        model: ModelRef::Id(Cow::Borrowed("PlatformBase")),
        coord: Vec3::new(20, 20, 20),
        dir: Direction::North,
        is_ground: false,
        is_ghost: false,
        variant_index: 0,
        color: Color::Default,
    };

    assert!(map.place_block(block.clone()));
    assert!(map.remove_block(&block));
    assert!(map.place_block(block))
}

#[test]
fn place_equivalent_ghost_block() {
    let mut map = Map::new();

    let block = Block {
        model: ModelRef::Id(Cow::Borrowed("PlatformBase")),
        coord: Vec3::new(20, 20, 20),
        dir: Direction::North,
        is_ground: false,
        is_ghost: true,
        variant_index: 0,
        color: Color::Default,
    };

    assert!(map.place_block(block.clone()));
    assert!(map.place_block(block))
}
