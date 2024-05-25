use std::{error::Error, path::Path};

use gamebox::{engines::game::map::BlockKind, Vec3};
use ordered_float::{FloatIsNan, NotNan};
use shared::{BlockDesc, ItemDesc, MapDesc, ModelId};

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
            match gbx_block.kind() {
                BlockKind::Normal(gbx_kind) => {
                    blocks.push(BlockDesc {
                        model_id: ModelId::Game {
                            name: gbx_block.id().to_owned(),
                        },
                        coord: gbx_kind.coord(),
                        dir: gbx_kind.direction(),
                        elem_color: gbx_block.elem_color(),
                    });
                }
                BlockKind::Free(..) => {
                    todo!()
                }
            }
        }

        let mut items = vec![];

        for gbx_item in gbx_map.items() {
            let rotation = gbx_item.rotation();

            items.push(ItemDesc {
                model_id: ModelId::Game {
                    name: gbx_item.id().to_owned(),
                },
                pos: vec3_f32_to_vec3_not_nan_f32(gbx_item.position())?,
                yaw: NotNan::new(rotation.yaw).unwrap(),
                pitch: NotNan::new(rotation.pitch).unwrap(),
                roll: NotNan::new(rotation.roll).unwrap(),
                pivot_pos: vec3_f32_to_vec3_not_nan_f32(gbx_item.pivot_position())?,
                elem_color: gbx_item.elem_color(),
                anim_offset: gbx_item.animation_offset(),
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

fn vec3_f32_to_vec3_not_nan_f32(vec: &Vec3<f32>) -> Result<Vec3<NotNan<f32>>, FloatIsNan> {
    Ok(Vec3 {
        x: NotNan::new(vec.x)?,
        y: NotNan::new(vec.y)?,
        z: NotNan::new(vec.z)?,
    })
}
