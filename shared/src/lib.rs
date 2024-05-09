use std::error::Error;

use gamebox::{
    engines::game::map::{Direction, ElemColor, PhaseOffset},
    Vec3,
};
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Framed, LengthDelimitedCodec};

pub type FramedTcpStream = Framed<TcpStream, LengthDelimitedCodec>;

pub fn framed_tcp_stream(tcp_stream: TcpStream) -> FramedTcpStream {
    LengthDelimitedCodec::new().framed(tcp_stream)
}

pub fn serialize(value: &impl Serialize) -> Result<Vec<u8>, Box<dyn Error>> {
    let bytes = bitcode::serialize(value)?;

    Ok(bytes)
}

pub fn deserialize<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, Box<dyn Error>> {
    let value = bitcode::deserialize(bytes)?;

    Ok(value)
}

#[derive(Serialize, Deserialize)]
pub struct MapDesc {
    pub custom_block_models: Vec<Vec<u8>>,
    pub custom_item_models: Vec<Vec<u8>>,
    pub blocks: Vec<BlockDesc>,
    pub items: Vec<ItemDesc>,
}

#[derive(Serialize, Deserialize)]
pub enum Message {
    PlaceBlock(BlockDesc),
    RemoveBlock(BlockDesc),
    PlaceItem(ItemDesc),
    RemoveItem(ItemDesc),
    AddBlockModel { bytes: Vec<u8> },
    AddItemModel { bytes: Vec<u8> },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct BlockDesc {
    pub model_id: ModelId,
    pub variant_index: u8,
    pub elem_color: ElemColor,
    pub kind: BlockDescKind,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum BlockDescKind {
    Normal {
        coordinate: Vec3<u8>,
        direction: Direction,
        is_ground: bool,
        is_ghost: bool,
    },
    Free {
        position: Vec3<NotNan<f32>>,
        yaw: NotNan<f32>,
        pitch: NotNan<f32>,
        roll: NotNan<f32>,
    },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ItemDesc {
    pub model_id: ModelId,
    pub position: Vec3<NotNan<f32>>,
    pub yaw: NotNan<f32>,
    pub pitch: NotNan<f32>,
    pub roll: NotNan<f32>,
    pub pivot_position: Vec3<NotNan<f32>>,
    pub elem_color: ElemColor,
    pub anim_offset: PhaseOffset,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ModelId {
    Game { name: String },
    Custom { hash: blake3::Hash },
}
