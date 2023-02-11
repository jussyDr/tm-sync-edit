use std::borrow::Cow;

use gbx::map::{Color, Direction};
use gbx::Vec3;
use tm_sync_edit_server::map::{Block, Map, ModelRef};

fn fill_map() {
    let mut map = Map::new();

    for x in 0..48 {
        for y in 10..40 {
            for z in 0..48 {
                map.place_block(Block {
                    model: ModelRef::Id(Cow::Borrowed("RoadTechBranchCross")),
                    coord: Vec3 { x, y, z },
                    dir: Direction::North,
                    is_ground: false,
                    is_ghost: false,
                    variant_index: 0,
                    color: Color::Default,
                })
                .unwrap();
            }
        }
    }
}

iai::main!(fill_map);
