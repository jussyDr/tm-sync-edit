use crate::serde;
use ::serde::ser::SerializeStruct;
use ::serde::{Deserialize, Serialize, Serializer};
use anyhow::anyhow;
use flat_multimap::FlatMultiset;
use gbx::map::{Color, Direction, PhaseOffset};
use gbx::Vec3;
use lazy_static::lazy_static;
use ordered_float::OrderedFloat;
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};
use zip::ZipArchive;

#[derive(Clone, Debug, Deserialize)]
pub enum BlockInfoClip {
    NonExclusive,
    ExclusiveSymmetric { id: String },
    ExclusiveAsymmetric { id: String, asym_clip_id: String },
}

impl BlockInfoClip {
    fn clips(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NonExclusive, Self::NonExclusive) => false,
            (Self::ExclusiveSymmetric { id }, Self::ExclusiveSymmetric { id: other_id }) => {
                id != other_id
            }
            (
                Self::ExclusiveAsymmetric { id, asym_clip_id },
                Self::ExclusiveAsymmetric {
                    id: other_id,
                    asym_clip_id: other_asym_clip_id,
                },
            ) => id != other_asym_clip_id || asym_clip_id != other_id,
            _ => true,
        }
    }
}

#[derive(Debug, Deserialize)]
struct UnitClips {
    clip_north: Option<BlockInfoClip>,
    clip_east: Option<BlockInfoClip>,
    clip_south: Option<BlockInfoClip>,
    clip_west: Option<BlockInfoClip>,
}

impl UnitClips {
    fn clip(&self, dir: Direction) -> Option<&BlockInfoClip> {
        match dir {
            Direction::North => self.clip_north.as_ref(),
            Direction::East => self.clip_east.as_ref(),
            Direction::South => self.clip_south.as_ref(),
            Direction::West => self.clip_west.as_ref(),
        }
    }
}

#[derive(Deserialize)]
struct BlockUnitInfo {
    offset: Vec3<u8>,
    clips: UnitClips,
}

#[derive(Deserialize)]
pub struct BlockInfoVariant {
    extent: Vec3<u8>,
    units: Vec<BlockUnitInfo>,
}

#[derive(Deserialize)]
struct BlockInfo {
    variants_ground: Vec<BlockInfoVariant>,
    variants_air: Vec<BlockInfoVariant>,
}

impl BlockInfo {
    fn variant(&self, ground: bool, index: u8) -> Option<&BlockInfoVariant> {
        if ground {
            self.variants_ground.get(index as usize)
        } else {
            self.variants_air.get(index as usize)
        }
    }
}

lazy_static! {
    static ref BLOCK_INFOS: HashMap<&'static str, BlockInfo> =
        serde_json::from_str(include_str!("BlockInfos.json")).unwrap();
    static ref ITEM_MODEL_IDS: HashSet<&'static str> =
        serde_json::from_str(include_str!("ItemModelIds.json")).unwrap();
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelRef {
    Id(Cow<'static, str>),
    Hash(serde::Base64<[u8; 32]>),
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Block {
    pub model: ModelRef,
    pub coord: Vec3<u8>,
    pub dir: Direction,
    pub is_ground: bool,
    pub variant_index: u8,
    pub is_ghost: bool,
    pub color: Color,
}

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FreeBlock {
    model: ModelRef,
    pos: Vec3<OrderedFloat<f32>>,
    yaw: OrderedFloat<f32>,
    pitch: OrderedFloat<f32>,
    roll: OrderedFloat<f32>,
    color: Color,
}

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Item {
    model: ModelRef,
    pos: Vec3<OrderedFloat<f32>>,
    yaw: OrderedFloat<f32>,
    pitch: OrderedFloat<f32>,
    roll: OrderedFloat<f32>,
    pivot_pos: Vec3<OrderedFloat<f32>>,
    color: Color,
    anim_offset: PhaseOffset,
}

#[allow(clippy::type_complexity)]
pub struct Map {
    size: Vec3<u8>,
    blocks: FlatMultiset<Block, ahash::RandomState>,
    units: HashMap<Vec3<u8>, UnitClips, ahash::RandomState>,
    free_blocks: FlatMultiset<FreeBlock, ahash::RandomState>,
    items: FlatMultiset<Item, ahash::RandomState>,
    embedded_blocks: HashMap<serde::Base64<[u8; 32]>, (&'static str, serde::Base64<Vec<u8>>)>,
    embedded_items: HashMap<serde::Base64<[u8; 32]>, serde::Base64<Vec<u8>>>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            size: Vec3 {
                x: 48,
                y: 40,
                z: 48,
            },
            blocks: FlatMultiset::with_hasher(ahash::RandomState::new()),
            units: HashMap::with_hasher(ahash::RandomState::new()),
            free_blocks: FlatMultiset::with_hasher(ahash::RandomState::new()),
            items: FlatMultiset::with_hasher(ahash::RandomState::new()),
            embedded_blocks: HashMap::new(),
            embedded_items: HashMap::new(),
        }
    }

    fn get_block_info(&self, model_ref: &ModelRef) -> Option<&'static BlockInfo> {
        let block_info_id = match *model_ref {
            ModelRef::Id(ref id) => id.as_ref(),
            ModelRef::Hash(ref hash) => self
                .embedded_blocks
                .get(hash)
                .map(|(archetype, _)| *archetype)?,
        };

        BLOCK_INFOS.get(block_info_id)
    }

    pub fn can_place_clip(&self, clip: &BlockInfoClip, coord: Vec3<u8>, dir: Direction) -> bool {
        let other_coord = match dir {
            Direction::North => {
                if coord.z < self.size.z - 1 {
                    coord + Vec3::new(0, 0, 1)
                } else {
                    return true;
                }
            }
            Direction::East => {
                if coord.x > 0 {
                    coord - Vec3::new(1, 0, 0)
                } else {
                    return true;
                }
            }
            Direction::South => {
                if coord.z > 0 {
                    coord - Vec3::new(0, 0, 1)
                } else {
                    return true;
                }
            }
            Direction::West => {
                if coord.x < self.size.x - 1 {
                    coord + Vec3::new(1, 0, 0)
                } else {
                    return true;
                }
            }
        };

        if let Some(other_clip) = self
            .units
            .get(&other_coord)
            .and_then(|clips| clips.clip(dir.opposite()))
        {
            if clip.clips(other_clip) {
                return false;
            }
        }

        true
    }
}

fn rotate_unit_offset(coord: Vec3<u8>, dir: Direction, extent: Vec3<u8>) -> Vec3<u8> {
    match dir {
        Direction::North => coord,
        Direction::East => Vec3 {
            x: extent.z - coord.z,
            y: coord.y,
            z: coord.x,
        },
        Direction::South => Vec3 {
            x: extent.x - coord.x,
            y: coord.y,
            z: extent.z - coord.z,
        },
        Direction::West => Vec3 {
            x: coord.z,
            y: coord.y,
            z: extent.x - coord.x,
        },
    }
}

impl Map {
    pub fn can_place_block(&self, block: &Block, variant: &BlockInfoVariant) -> bool {
        for unit_info in &variant.units {
            let coord =
                block.coord + rotate_unit_offset(unit_info.offset, block.dir, variant.extent);

            if self.units.contains_key(&coord) {
                return false;
            }

            for dir in [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ] {
                if let Some(clip) = &unit_info.clips.clip(dir) {
                    if !self.can_place_clip(clip, coord, dir + block.dir) {
                        return false;
                    }
                }
            }
        }

        true
    }

    pub fn place_block(&mut self, block: Block) -> bool {
        let block_info = if let Some(block_info) = self.get_block_info(&block.model) {
            block_info
        } else {
            return false;
        };

        let variant =
            if let Some(variant) = block_info.variant(block.is_ground, block.variant_index) {
                variant
            } else {
                return false;
            };

        let extent = block.coord + variant.extent;

        if extent.x >= self.size.x || extent.y >= self.size.y || extent.z >= self.size.z {
            return false;
        }

        if !block.is_ghost {
            if !self.can_place_block(&block, variant) {
                return false;
            }

            for unit_info in &variant.units {
                let coord =
                    block.coord + rotate_unit_offset(unit_info.offset, block.dir, variant.extent);

                self.units.insert(
                    coord,
                    UnitClips {
                        clip_north: unit_info.clips.clip(Direction::North - block.dir).cloned(),
                        clip_east: unit_info.clips.clip(Direction::East - block.dir).cloned(),
                        clip_south: unit_info.clips.clip(Direction::South - block.dir).cloned(),
                        clip_west: unit_info.clips.clip(Direction::West - block.dir).cloned(),
                    },
                );
            }
        }

        self.blocks.insert(block);

        true
    }

    pub fn remove_block(&mut self, block: &Block) -> bool {
        if self.blocks.remove(block) {
            if !block.is_ghost {
                let block_info = self.get_block_info(&block.model).unwrap();

                let variant = block_info
                    .variant(block.is_ground, block.variant_index)
                    .unwrap();

                for unit_info in &variant.units {
                    let coord = block.coord
                        + rotate_unit_offset(unit_info.offset, block.dir, variant.extent);

                    self.units.remove(&coord);
                }
            }

            true
        } else {
            false
        }
    }

    pub fn place_free_block(&mut self, free_block: FreeBlock) -> bool {
        if self.get_block_info(&free_block.model).is_none() {
            return false;
        }

        self.free_blocks.insert(free_block);

        true
    }

    pub fn remove_free_block(&mut self, free_block: &FreeBlock) -> bool {
        self.free_blocks.remove(free_block)
    }

    pub fn place_item(&mut self, item: Item) -> bool {
        let known_item_model = match item.model {
            ModelRef::Id(ref id) => ITEM_MODEL_IDS.contains(id.as_ref()),
            ModelRef::Hash(ref hash) => self.embedded_items.contains_key(hash),
        };

        if !known_item_model {
            return false;
        }

        self.items.insert(item);

        true
    }

    pub fn remove_item(&mut self, item: &Item) -> bool {
        self.items.remove(item)
    }

    pub fn load<R>(reader: R) -> anyhow::Result<Self>
    where
        R: Read,
    {
        let gbx_map = gbx::Map::reader().read_from(reader)?;

        let mut map = Map::new();

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
                    let block = gbx::Block::reader().read_from(bytes.as_slice())?;

                    let (archetype, _) = BLOCK_INFOS
                        .get_key_value(block.archetype.as_str())
                        .ok_or_else(|| anyhow!("unknown block archetype"))?;

                    map.embedded_blocks
                        .insert(hash.into(), (archetype, bytes.into()));
                } else if path_lowercase.ends_with(".item.gbx") {
                    map.embedded_items.insert(hash.into(), bytes.into());
                } else {
                    return Err(anyhow!("unknown embedded file extension: {path}"));
                }

                embedded_files.insert(path, hash);
            }
        }

        for gbx_block in gbx_map.blocks {
            let model_id = gbx_block.model_id();

            let model = BLOCK_INFOS
                .get_key_value(model_id.as_str())
                .map(|(model_id, _)| ModelRef::Id(Cow::Borrowed(model_id)))
                .or_else(|| {
                    embedded_files
                        .get(model_id.strip_suffix("_CustomBlock").unwrap())
                        .map(|&hash| ModelRef::Hash(hash.into()))
                })
                .ok_or_else(|| anyhow!("unknown block model id: {}", model_id))?;

            match gbx_block {
                gbx::map::BlockType::Normal(gbx_block) => {
                    map.place_block(Block {
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
                    map.place_free_block(FreeBlock {
                        model,
                        pos: Vec3 {
                            x: gbx_free_block.pos.x.into(),
                            y: gbx_free_block.pos.y.into(),
                            z: gbx_free_block.pos.z.into(),
                        },
                        yaw: gbx_free_block.yaw.into(),
                        pitch: gbx_free_block.pitch.into(),
                        roll: gbx_free_block.roll.into(),
                        color: gbx_free_block.color,
                    });
                }
            }
        }

        for gbx_item in gbx_map.items {
            let model = ITEM_MODEL_IDS
                .get(gbx_item.model_id.as_str())
                .map(|model_id| ModelRef::Id(Cow::Borrowed(model_id)))
                .or_else(|| {
                    embedded_files
                        .get(gbx_item.model_id.as_str())
                        .map(|&hash| ModelRef::Hash(hash.into()))
                })
                .ok_or_else(|| anyhow!("unknown item model id: {}", gbx_item.model_id))?;

            map.place_item(Item {
                model,
                pos: Vec3 {
                    x: gbx_item.pos.x.into(),
                    y: gbx_item.pos.y.into(),
                    z: gbx_item.pos.z.into(),
                },
                yaw: gbx_item.yaw.into(),
                pitch: gbx_item.pitch.into(),
                roll: gbx_item.roll.into(),
                pivot_pos: Vec3 {
                    x: gbx_item.pivot_pos.x.into(),
                    y: gbx_item.pivot_pos.y.into(),
                    z: gbx_item.pivot_pos.z.into(),
                },
                color: gbx_item.color,
                anim_offset: gbx_item.anim_offset,
            });
        }

        Ok(map)
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_struct("Map", 6)?;
        map.serialize_field("size", &self.size)?;
        map.serialize_field("blocks", &self.blocks)?;
        map.serialize_field("free_blocks", &self.free_blocks)?;
        map.serialize_field("items", &self.items)?;
        map.serialize_field("embedded_blocks", &self.embedded_blocks)?;
        map.serialize_field("embedded_items", &self.embedded_items)?;
        map.end()
    }
}

#[cfg(test)]
mod tests {
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

        let variant =
            if let Some(variant) = block_info.variant(block.is_ground, block.variant_index) {
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
}
