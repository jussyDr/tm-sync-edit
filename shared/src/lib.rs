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
pub struct MapParamsDesc;

#[derive(Serialize, Deserialize)]
pub struct MapDesc;
