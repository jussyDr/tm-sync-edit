pub mod map;

mod serde;

use ::serde::{Deserialize, Serialize};
use futures_util::{SinkExt, TryStreamExt};
use map::{Block, FreeBlock, Item};
use std::io::Result;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub struct Server;

impl Server {
    pub async fn run() -> Result<()> {
        let ip = Ipv4Addr::UNSPECIFIED;
        let port = 8369;
        let addr = SocketAddrV4::new(ip, port);
        let listener = TcpListener::bind(addr).await?;

        tracing::info!("listening on {addr}");

        loop {
            let (stream, addr) = listener.accept().await?;

            tokio::spawn(async move {
                tracing::debug!("accepted connection from {addr}");

                let framed_stream = LengthDelimitedCodec::builder()
                    .little_endian()
                    .length_field_type::<u32>()
                    .new_framed(stream);

                if let Err(err) = handle_client(framed_stream).await {
                    tracing::error!("error: {}", err);
                }

                tracing::debug!("closed connection to {addr}");
            });
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    PlaceBlock(Block),
    RemoveBlock(Block),
    PlaceFreeBlock(FreeBlock),
    RemoveFreeBlock(FreeBlock),
    PlaceItem(Item),
    RemoveItem(Item),
}

async fn handle_client(mut stream: Framed<TcpStream, LengthDelimitedCodec>) -> anyhow::Result<()> {
    loop {
        tokio::select! {
            result = stream.try_next() => match result? {
                Some(frame) => {
                    let command: Command = serde_json::from_slice(&frame)?;

                    println!("{command:?}");

                    match command {
                        Command::PlaceBlock(block) => {
                            let response = Command::RemoveBlock(block);
                            let frame = serde_json::to_vec(&response)?;

                            stream.send(frame.into()).await?;
                        }
                        Command::RemoveBlock(block) => {
                            let response = Command::PlaceBlock(block);
                            let frame = serde_json::to_vec(&response)?;

                            stream.send(frame.into()).await?;
                        }
                        Command::PlaceFreeBlock(free_block) => {
                            let response = Command::RemoveFreeBlock(free_block);
                            let frame = serde_json::to_vec(&response)?;

                            stream.send(frame.into()).await?;
                        }
                        Command::RemoveFreeBlock(free_block) => {
                            let response = Command::PlaceFreeBlock(free_block);
                            let frame = serde_json::to_vec(&response)?;

                            stream.send(frame.into()).await?;
                        }
                        Command::PlaceItem(item) => {
                            let response = Command::RemoveItem(item);
                            let frame = serde_json::to_vec(&response)?;

                            stream.send(frame.into()).await?;
                        }
                        Command::RemoveItem(item) => {
                            let response = Command::PlaceItem(item);
                            let frame = serde_json::to_vec(&response)?;

                            stream.send(frame.into()).await?;
                        }
                    }
                },
                None => break,
            }
        }
    }

    Ok(())
}
