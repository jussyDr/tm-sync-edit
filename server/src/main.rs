//! Trackmania Sync Edit server.

use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    thread,
};

use anyhow::Result;
use bytes::Bytes;
use clap::Parser;
use futures_util::{SinkExt, TryStreamExt};
use log::LevelFilter;
use tm_sync_edit_shared::{deserialize, framed_tcp_stream, FramedTcpStream, Map, Message};
use tokio::{
    net::{TcpListener, TcpStream},
    runtime, select, spawn,
    sync::{mpsc, Mutex},
};

/// Command line arguments.
#[derive(clap::Parser)]
struct Args {
    #[clap(short, long)]
    port: Option<u16>,
    #[clap(short, long)]
    num_threads: Option<usize>,
}

fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let args = Args::parse();

    let num_threads = args
        .num_threads
        .unwrap_or(thread::available_parallelism()?.get());

    log::info!("Number of threads: {num_threads}");

    let mut runtime_builder = if num_threads == 1 {
        runtime::Builder::new_current_thread()
    } else {
        runtime::Builder::new_multi_thread()
    };

    let runtime = runtime_builder.enable_io().build()?;

    runtime.block_on(async {
        let state = Arc::new(Mutex::new(State::new()));

        let ip_addr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
        let port = args.port.unwrap_or(8369);
        let socket_addr = SocketAddr::new(ip_addr, port);

        let tcp_listener = TcpListener::bind(socket_addr).await?;

        log::info!("Listening on: {socket_addr}");

        loop {
            let (tcp_stream, socket_addr) = tcp_listener.accept().await?;

            let state = Arc::clone(&state);

            spawn(async move {
                log::info!("Connection from: {socket_addr}");

                if let Err(error) = handle_connection(state, tcp_stream, socket_addr).await {
                    log::error!("{error}")
                }
            });
        }
    })
}

async fn handle_connection(
    state: Arc<Mutex<State>>,
    tcp_stream: TcpStream,
    socket_addr: SocketAddr,
) -> Result<()> {
    let mut framed_tcp_stream = framed_tcp_stream(tcp_stream);

    let (sender, receiver) = mpsc::unbounded_channel();

    let frame = {
        let mut state = state.lock().await;

        state.clients.insert(socket_addr, sender);

        tm_sync_edit_shared::serialize(&state.map)?
    };

    framed_tcp_stream.send(Bytes::from(frame)).await?;

    let result = handle_client(framed_tcp_stream, receiver).await;

    state.lock().await.clients.remove(&socket_addr);

    log::info!("Disconnected: {socket_addr}");

    result
}

async fn handle_client(
    mut framed_tcp_stream: FramedTcpStream,
    mut receiver: mpsc::UnboundedReceiver<Bytes>,
) -> Result<()> {
    loop {
        select! {
            result = framed_tcp_stream.try_next() => match result? {
                None => break,
                Some(frame) => {
                    let frame = frame.freeze();

                    let message: Message = deserialize(&frame)?;

                    match message {
                        Message::PlaceBlock => {}
                        Message::RemoveBlock => {}
                        Message::PlaceItem => {}
                        Message::RemoveItem => {}
                        Message::AddCustomBlockInfo => {}
                        Message::AddCustomItemModel => {}
                    }
                }
            },
            frame = receiver.recv() => match frame {
                None => todo!(),
                Some(frame) => {
                    framed_tcp_stream.send(frame).await?;
                }
            }
        }
    }

    Ok(())
}

struct State {
    clients: HashMap<SocketAddr, mpsc::UnboundedSender<Bytes>>,
    map: Map,
}

impl State {
    fn new() -> Self {
        Self {
            clients: HashMap::new(),
            map: Map::default(),
        }
    }
}
