use std::{error::Error, path::Path};

use gamebox::engines::game::map::BlockKind;
use ordered_float::NotNan;
use shared::{BlockDesc, BlockDescKind, Direction, ElemColor, ItemDesc, MapDesc, ModelId};

pub struct Map {
    pub desc: MapDesc,
}

impl Map {
    pub fn new() -> Self {
        Self {
            desc: MapDesc {
                custom_block_models: vec![],
                custom_item_models: vec![],
                blocks: vec![],
                items: vec![],
            },
        }
    }

    pub fn load_from_gbx(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let gbx_map: gamebox::Map = gamebox::read_file(path)?;

        if let Some(embedded_objects) = gbx_map.embedded_objects() {
            if !embedded_objects.ids().is_empty() {
                todo!()
            }
        }

        let mut blocks = vec![];

        for gbx_block in gbx_map.blocks() {
            let kind = match gbx_block.kind() {
                BlockKind::Normal(gbx_kind) => {
                    let coord = gbx_kind.coord().as_array();

                    BlockDescKind::Normal {
                        x: coord[0],
                        y: coord[1],
                        z: coord[2],
                        direction: direction_from_gbx_direction(gbx_kind.direction()),
                        is_ground: gbx_kind.is_ground(),
                        is_ghost: gbx_kind.is_ghost(),
                    }
                }
                BlockKind::Free(gbx_block_kind) => {
                    let position = gbx_block_kind.position().as_array();
                    let rotation = gbx_block_kind.rotation().as_array();

                    BlockDescKind::Free {
                        x: NotNan::new(position[0]).unwrap(),
                        y: NotNan::new(position[1]).unwrap(),
                        z: NotNan::new(position[2]).unwrap(),
                        yaw: NotNan::new(rotation[0]).unwrap(),
                        pitch: NotNan::new(rotation[1]).unwrap(),
                        roll: NotNan::new(rotation[2]).unwrap(),
                    }
                }
            };

            blocks.push(BlockDesc {
                model_id: ModelId::Game {
                    name: gbx_block.id().to_owned(),
                },
                elem_color: elem_color_from_gbx_elem_color(gbx_block.elem_color()),
                kind,
            });
        }

        let mut items = vec![];

        for gbx_item in gbx_map.items() {
            let position = gbx_item.position().as_array();
            let rotation = gbx_item.rotation().as_array();

            items.push(ItemDesc {
                model_id: ModelId::Game {
                    name: gbx_item.id().to_owned(),
                },
                x: NotNan::new(position[0]).unwrap(),
                y: NotNan::new(position[1]).unwrap(),
                z: NotNan::new(position[2]).unwrap(),
                yaw: NotNan::new(rotation[0]).unwrap(),
                pitch: NotNan::new(rotation[1]).unwrap(),
                roll: NotNan::new(rotation[2]).unwrap(),
                elem_color: elem_color_from_gbx_elem_color(gbx_item.elem_color()),
            })
        }

        Ok(Self {
            desc: MapDesc {
                custom_block_models: vec![],
                custom_item_models: vec![],
                blocks,
                items,
            },
        })
    }
}

fn direction_from_gbx_direction(
    gbx_direction: gamebox::engines::game::map::Direction,
) -> Direction {
    match gbx_direction {
        gamebox::engines::game::map::Direction::North => Direction::North,
        gamebox::engines::game::map::Direction::East => Direction::East,
        gamebox::engines::game::map::Direction::South => Direction::South,
        gamebox::engines::game::map::Direction::West => Direction::West,
    }
}

fn elem_color_from_gbx_elem_color(
    gbx_elem_color: gamebox::engines::game::map::ElemColor,
) -> ElemColor {
    match gbx_elem_color {
        gamebox::engines::game::map::ElemColor::Default => ElemColor::Default,
        gamebox::engines::game::map::ElemColor::White => ElemColor::White,
        gamebox::engines::game::map::ElemColor::Green => ElemColor::Green,
        gamebox::engines::game::map::ElemColor::Blue => ElemColor::Blue,
        gamebox::engines::game::map::ElemColor::Red => ElemColor::Red,
        gamebox::engines::game::map::ElemColor::Black => ElemColor::Black,
    }
}