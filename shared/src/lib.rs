//! Items shared between the server and client.

use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub type FramedTcpStream = Framed<TcpStream, LengthDelimitedCodec>;

pub fn framed_tcp_stream(tcp_stream: TcpStream) -> FramedTcpStream {
    LengthDelimitedCodec::builder().new_framed(tcp_stream)
}

pub fn serialize(value: &impl Serialize) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_stdvec(value)
}

pub fn deserialize<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, postcard::Error> {
    postcard::from_bytes(bytes)
}

#[derive(Default, Serialize, Deserialize)]
pub struct Map {
    pub blocks: Vec<()>,
    pub items: Vec<()>,
    pub custom_block_infos: Vec<()>,
    pub custom_item_models: Vec<()>,
}

#[derive(Serialize, Deserialize)]
pub enum Message {
    PlaceBlock,
    RemoveBlock,
    PlaceItem,
    RemoveItem,
    AddCustomBlockInfo,
    AddCustomItemModel,
}
