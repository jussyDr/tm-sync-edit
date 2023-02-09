mod map;
mod serde;

use bytes::Bytes;
use futures_util::SinkExt;
use map::Map;
use std::fs::File;
use std::io::Result;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::TcpListener;
use tokio_util::codec::LengthDelimitedCodec;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    let level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };

    tracing_subscriber::fmt().with_max_level(level).init();

    let ip = Ipv4Addr::UNSPECIFIED;
    let port = 8369;
    let addr = SocketAddrV4::new(ip, port);
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("listening on {addr}");

    loop {
        let (stream, addr) = listener.accept().await?;

        tokio::spawn(async move {
            tracing::debug!("accepted connection from {addr}");

            let mut framed_stream = LengthDelimitedCodec::builder()
                .little_endian()
                .length_field_type::<u32>()
                .new_framed(stream);

            let map =
                Map::load(File::open("Gammax 2 - 8 punten & 3 strepen.Map.Gbx").unwrap()).unwrap();
            let bytes: Bytes = serde_json::to_vec(&map).unwrap().into();
            framed_stream.send(bytes).await.unwrap();

            tracing::debug!("closed connection to {addr}");
        });
    }
}
