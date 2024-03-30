use std::error::Error;

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
    PlaceBlock { block: Block },
    RemoveBlock { block: Block },
    PlaceGhostBlock { ghost_block: GhostBlock },
    RemoveGhostBlock { ghost_block: GhostBlock },
    PlaceFreeBlock { free_block: FreeBlock },
    RemoveFreeBlock { free_block: FreeBlock },
    PlaceItem { item: Item },
    RemoveItem { item: Item },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub direction: Direction,
    pub elem_color: ElemColor,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ElemColor {
    Default,
    White,
    Green,
    Blue,
    Red,
    Black,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GhostBlock;

#[derive(Debug, Serialize, Deserialize)]
pub struct FreeBlock;

#[derive(Debug, Serialize, Deserialize)]
pub struct Item;
