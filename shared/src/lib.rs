use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Framed, LengthDelimitedCodec};

pub type FramedTcpStream = Framed<TcpStream, LengthDelimitedCodec>;

pub fn framed_tcp_stream(tcp_stream: TcpStream) -> FramedTcpStream {
    LengthDelimitedCodec::new().framed(tcp_stream)
}

pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_stdvec(&value)
}

pub fn deserialize<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, postcard::Error> {
    postcard::from_bytes(bytes)
}

#[derive(Serialize, Deserialize)]
pub enum Mood {
    Day,
    Sunset,
    Night,
    Sunrise,
}

#[derive(Serialize, Deserialize)]
pub struct MapParamsDesc {
    pub mood: Mood,
}

#[derive(Serialize, Deserialize)]
pub struct MapDesc {
    pub blocks: Vec<BlockDesc>,
}

#[derive(Serialize, Deserialize)]
pub struct BlockDesc {
    pub block_info_name: String,
    pub x: u8,
    pub y: u8,
    pub z: u8,
}
