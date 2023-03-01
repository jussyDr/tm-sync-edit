pub mod map;

mod serde;

use ::serde::{Deserialize, Serialize};
use bytes::Bytes;
use futures_util::{SinkExt, TryStreamExt};
use map::{Block, FreeBlock, Item, Map, PlaceBlockError};
use std::collections::HashMap;
use std::io::Result;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub struct Server {
    clients: HashMap<SocketAddr, mpsc::UnboundedSender<Bytes>>,
    map: Map,
}

impl Server {
    pub async fn run() -> Result<()> {
        let server = Arc::new(Mutex::new(Server {
            clients: HashMap::new(),
            map: Map::new(),
        }));

        let ip = Ipv4Addr::UNSPECIFIED;
        let port = 8369;
        let addr = SocketAddrV4::new(ip, port);
        let listener = TcpListener::bind(addr).await?;

        tracing::info!("listening on {addr}");

        loop {
            let (stream, addr) = listener.accept().await?;
            let server = Arc::clone(&server);

            tokio::spawn(async move {
                tracing::debug!("accepted connection from {addr}");

                if let Err(err) = handle_connection(&server, stream, addr).await {
                    tracing::error!("error: {}", err);
                }

                tracing::debug!("closed connection to {addr}");
            });
        }
    }
}

#[derive(Debug, Deserialize)]
enum ClientCommand {
    PlaceBlock(String),
    RemoveBlock(String),
    PlaceGhostBlock(String),
    RemoveGhostBlock(String),
    PlaceFreeBlock(String),
    RemoveFreeBlock(String),
    PlaceItem(String),
    RemoveItem(String),
}

#[derive(Serialize)]
enum ServerCommand {
    PlaceBlock(String),
    RemoveBlock(String),
    SetGhostBlockCount {
        block_json: String,
        count: usize,
    },
    SetFreeBlockCount {
        free_block_json: String,
        count: usize,
    },
    SetItemCount {
        item_json: String,
        count: usize,
    },
}

async fn handle_connection(
    server: &Mutex<Server>,
    stream: TcpStream,
    addr: SocketAddr,
) -> anyhow::Result<()> {
    let framed_stream = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_type::<u32>()
        .new_framed(stream);

    let (sender, receiver) = mpsc::unbounded_channel();

    server.lock().await.clients.insert(addr, sender);

    if let Err(err) = handle_client(server, framed_stream, receiver).await {
        tracing::error!("error: {}", err);
    }

    server.lock().await.clients.remove(&addr);

    Ok(())
}

async fn handle_client(
    server: &Mutex<Server>,
    mut stream: Framed<TcpStream, LengthDelimitedCodec>,
    mut receiver: mpsc::UnboundedReceiver<Bytes>,
) -> anyhow::Result<()> {
    loop {
        tokio::select! {
            result = stream.try_next() => match result? {
                Some(frame) => {
                    let frame = frame.freeze();
                    let command: ClientCommand = serde_json::from_slice(&frame)?;

                    match command {
                        ClientCommand::PlaceBlock(block_json) => {
                            let block: Block = serde_json::from_str(&block_json)?;

                            let mut server = server.lock().await;

                            match server.map.place_block(block) {
                                Ok(()) => {
                                    let command = ServerCommand::PlaceBlock(block_json);
                                    let frame: Bytes = serde_json::to_vec(&command)?.into();

                                    for client in server.clients.values() {
                                        client.send(Bytes::clone(&frame))?;
                                    }
                                }
                                Err(PlaceBlockError::Failed) => {
                                    let command = ServerCommand::RemoveBlock(block_json);
                                    let frame = serde_json::to_vec(&command)?;
                                    stream.send(frame.into()).await?;
                                }
                                Err(PlaceBlockError::Occupied) => {}
                            }
                        }
                        ClientCommand::RemoveBlock(block_json) => {
                            let block: Block = serde_json::from_str(&block_json)?;

                            let mut server = server.lock().await;

                            if server.map.remove_block(&block) {
                                let command = ServerCommand::RemoveBlock(block_json);
                                let frame: Bytes = serde_json::to_vec(&command)?.into();

                                for client in server.clients.values() {
                                    client.send(Bytes::clone(&frame))?;
                                }
                            }
                        }
                        ClientCommand::PlaceGhostBlock(block_json) => {
                            let block: Block = serde_json::from_str(&block_json)?;

                            let mut server = server.lock().await;
                            let (ok, count) = server.map.place_ghost_block(block);

                            let command = ServerCommand::SetGhostBlockCount { block_json, count };
                            let frame: Bytes = serde_json::to_vec(&command)?.into();

                            if ok {
                                for client in server.clients.values() {
                                    client.send(Bytes::clone(&frame))?;
                                }
                            } else {
                                stream.send(frame).await?;
                            }
                        }
                        ClientCommand::RemoveGhostBlock(block_json) => {
                            let block: Block = serde_json::from_str(&block_json)?;

                            let mut server = server.lock().await;
                            let (ok, count) = server.map.remove_ghost_block(&block);

                            if ok {
                                let command = ServerCommand::SetGhostBlockCount { block_json, count };
                                let frame: Bytes = serde_json::to_vec(&command)?.into();

                                for client in server.clients.values() {
                                    client.send(Bytes::clone(&frame))?;
                                }
                            }
                        }
                        ClientCommand::PlaceFreeBlock(free_block_json) => {
                            let free_block: FreeBlock = serde_json::from_str(&free_block_json)?;

                            let mut server = server.lock().await;
                            let (ok, count) = server.map.place_free_block(free_block);

                            let command = ServerCommand::SetFreeBlockCount { free_block_json, count };
                            let frame: Bytes = serde_json::to_vec(&command)?.into();

                            if ok {
                                for client in server.clients.values() {
                                    client.send(Bytes::clone(&frame))?;
                                }
                            } else {
                                stream.send(frame).await?;
                            }
                        }
                        ClientCommand::RemoveFreeBlock(free_block_json) => {
                            let free_block: FreeBlock = serde_json::from_str(&free_block_json)?;

                            let mut server = server.lock().await;
                            let (ok, count) = server.map.remove_free_block(&free_block);

                            if ok {
                                let command = ServerCommand::SetFreeBlockCount { free_block_json, count };
                                let frame: Bytes = serde_json::to_vec(&command)?.into();

                                for client in server.clients.values() {
                                    client.send(Bytes::clone(&frame))?;
                                }
                            }
                        }
                        ClientCommand::PlaceItem(item_json) => {
                            let item: Item = serde_json::from_str(&item_json)?;

                            let mut server = server.lock().await;
                            let (ok, count) = server.map.place_item(item);

                            let command = ServerCommand::SetItemCount { item_json, count };
                            let frame: Bytes = serde_json::to_vec(&command)?.into();

                            if ok {
                                for client in server.clients.values() {
                                    client.send(Bytes::clone(&frame))?;
                                }
                            } else {
                                stream.send(frame).await?;
                            }
                        }
                        ClientCommand::RemoveItem(item_json) => {
                            let item: Item = serde_json::from_str(&item_json)?;

                            let mut server = server.lock().await;
                            let (ok, count) = server.map.remove_item(&item);

                            if ok {
                                let command = ServerCommand::SetItemCount { item_json, count };
                                let frame: Bytes = serde_json::to_vec(&command)?.into();

                                for client in server.clients.values() {
                                    client.send(Bytes::clone(&frame))?;
                                }
                            }
                        }
                    }
                },
                None => break,
            },
            Some(frame) = receiver.recv() => {
                stream.send(frame).await?;
            }
        }
    }

    Ok(())
}
