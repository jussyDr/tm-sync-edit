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
    PlaceItem(ItemDesc),
    RemoveItem(ItemDesc),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct BlockDesc {
    pub block_info_name: String,
    pub block_info_is_custom: bool,
    pub elem_color: ElemColor,
    pub kind: BlockDescKind,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum BlockDescKind {
    Normal {
        x: u8,
        y: u8,
        z: u8,
        direction: Direction,
        is_ground: bool,
        is_ghost: bool,
    },
    Free {
        x: NotNan<f32>,
        y: NotNan<f32>,
        z: NotNan<f32>,
        yaw: NotNan<f32>,
        pitch: NotNan<f32>,
        roll: NotNan<f32>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum ElemColor {
    Default,
    White,
    Green,
    Blue,
    Red,
    Black,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
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
