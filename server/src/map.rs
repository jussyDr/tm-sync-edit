use crate::serde;
use ::serde::Serialize;
use anyhow::anyhow;
use lazy_static::lazy_static;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};
use zip::ZipArchive;

lazy_static! {
    static ref BLOCK_INFOS: HashMap<&'static str, ()> =
        serde_json::from_str(include_str!("BlockInfos.json")).unwrap();
    static ref ITEM_MODEL_IDS: HashSet<&'static str> =
        serde_json::from_str(include_str!("ItemModelIds.json")).unwrap();
}

#[derive(Serialize)]
enum ModelRef {
    Id(&'static str),
    Hash(serde::Base64<[u8; 32]>),
}

#[derive(Serialize)]
struct Block {
    model: ModelRef,
    coord: gbx::Vec3<u8>,
    dir: gbx::map::Direction,
    is_ground: bool,
    variant_index: u8,
    is_ghost: bool,
    color: gbx::map::Color,
}

#[derive(Serialize)]
struct FreeBlock {
    model: ModelRef,
    pos: gbx::Vec3<f32>,
    yaw: f32,
    pitch: f32,
    roll: f32,
    color: gbx::map::Color,
}

#[derive(Serialize)]
struct Item {
    model: ModelRef,
    pos: gbx::Vec3<f32>,
    yaw: f32,
    pitch: f32,
    roll: f32,
    pivot_pos: gbx::Vec3<f32>,
    color: gbx::map::Color,
    anim_offset: gbx::map::PhaseOffset,
}

#[derive(Serialize)]
pub struct Map {
    blocks: Vec<Block>,
    free_blocks: Vec<FreeBlock>,
    items: Vec<Item>,
    embedded_blocks: HashMap<serde::Base64<[u8; 32]>, serde::Base64<Vec<u8>>>,
    embedded_items: HashMap<serde::Base64<[u8; 32]>, serde::Base64<Vec<u8>>>,
}

impl Map {
    pub fn load<R>(reader: R) -> anyhow::Result<Self>
    where
        R: Read,
    {
        let gbx_map = gbx::Map::reader().read_from(reader)?;

        let mut embedded_blocks = HashMap::new();
        let mut embedded_items = HashMap::new();
        let mut embedded_files = HashMap::new();

        if let Some(gbx_embedded_files) = gbx_map.embedded_files {
            let mut archive = ZipArchive::new(Cursor::new(gbx_embedded_files.archive))?;

            for (i, path) in gbx_embedded_files.paths.into_iter().enumerate() {
                let mut file = archive.by_index(i)?;
                let mut bytes = Vec::with_capacity(file.size() as usize);
                file.read_to_end(&mut bytes)?;

                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                let hash: [u8; 32] = hasher.finalize().into();

                let path_lowercase = path.to_ascii_lowercase();

                if path_lowercase.ends_with(".block.gbx") {
                    embedded_blocks.insert(hash.into(), bytes.into());
                } else if path_lowercase.ends_with(".item.gbx") {
                    embedded_items.insert(hash.into(), bytes.into());
                } else {
                    return Err(anyhow!("unknown embedded file extension: {path}"));
                }

                embedded_files.insert(path, hash);
            }
        }

        let mut blocks = vec![];
        let mut free_blocks = vec![];

        for gbx_block in gbx_map.blocks {
            let model_id = gbx_block.model_id();

            let model = BLOCK_INFOS
                .get_key_value(model_id.as_str())
                .map(|(model_id, _)| ModelRef::Id(model_id))
                .or_else(|| {
                    embedded_files
                        .get(model_id.strip_suffix("_CustomBlock").unwrap())
                        .map(|&hash| ModelRef::Hash(hash.into()))
                })
                .ok_or_else(|| anyhow!("unknown block model id: {}", model_id))?;

            match gbx_block {
                gbx::map::BlockType::Normal(gbx_block) => {
                    blocks.push(Block {
                        model,
                        coord: gbx_block.coord,
                        dir: gbx_block.dir,
                        is_ground: gbx_block.is_ground,
                        variant_index: gbx_block.variant_index,
                        is_ghost: gbx_block.is_ghost,
                        color: gbx_block.color,
                    });
                }
                gbx::map::BlockType::Free(gbx_free_block) => {
                    free_blocks.push(FreeBlock {
                        model,
                        pos: gbx_free_block.pos,
                        yaw: gbx_free_block.yaw,
                        pitch: gbx_free_block.pitch,
                        roll: gbx_free_block.roll,
                        color: gbx_free_block.color,
                    });
                }
            }
        }

        let mut items = vec![];

        for gbx_item in gbx_map.items {
            let model = ITEM_MODEL_IDS
                .get(gbx_item.model_id.as_str())
                .map(|model_id| ModelRef::Id(model_id))
                .or_else(|| {
                    embedded_files
                        .get(gbx_item.model_id.as_str())
                        .map(|&hash| ModelRef::Hash(hash.into()))
                })
                .ok_or_else(|| anyhow!("unknown item model id: {}", gbx_item.model_id))?;

            items.push(Item {
                model,
                pos: gbx_item.pos,
                yaw: gbx_item.yaw,
                pitch: gbx_item.pitch,
                roll: gbx_item.roll,
                pivot_pos: gbx_item.pivot_pos,
                color: gbx_item.color,
                anim_offset: gbx_item.anim_offset,
            });
        }

        Ok(Self {
            blocks,
            free_blocks,
            items,
            embedded_blocks,
            embedded_items,
        })
    }
}
