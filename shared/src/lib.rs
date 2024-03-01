use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Serialize, Deserialize)]
pub enum Message {
    PlaceBlock,
    RemoveBlock,
    PlaceItem,
    RemoveItem,
}

pub type FramedTcpStream = Framed<TcpStream, LengthDelimitedCodec>;

pub fn framed_tcp_stream(tcp_stream: TcpStream) -> FramedTcpStream {
    LengthDelimitedCodec::builder().new_framed(tcp_stream)
}

pub fn deserialize_message(bytes: &[u8]) -> Result<Message, postcard::Error> {
    postcard::from_bytes(bytes)
}
