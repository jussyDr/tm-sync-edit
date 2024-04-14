use std::error::Error;

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
pub enum Message {
    PlaceBlock(BlockDesc),
    RemoveBlock(BlockDesc),
    PlaceFreeBlock(FreeBlockDesc),
    RemoveFreeBlock(FreeBlockDesc),
    PlaceItem(ItemDesc),
    RemoveItem(ItemDesc),
}

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct BlockDesc {
    pub block_info_id: String,
    pub block_info_is_custom: bool,
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub direction: Direction,
    pub is_ground: bool,
    pub is_ghost: bool,
    pub elem_color: ElemColor,
}

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ElemColor {
    Default,
    White,
    Green,
    Blue,
    Red,
    Black,
}

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct FreeBlockDesc {
    pub block_info_id: String,
    pub block_info_is_custom: bool,
    pub x: NotNan<f32>,
    pub y: NotNan<f32>,
    pub z: NotNan<f32>,
    pub yaw: NotNan<f32>,
    pub pitch: NotNan<f32>,
    pub roll: NotNan<f32>,
    pub elem_color: ElemColor,
}

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ItemDesc {
    pub item_model_id: String,
    pub item_model_is_custom: bool,
    pub x: NotNan<f32>,
    pub y: NotNan<f32>,
    pub z: NotNan<f32>,
    pub yaw: NotNan<f32>,
    pub pitch: NotNan<f32>,
    pub roll: NotNan<f32>,
}
