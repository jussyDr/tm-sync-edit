use std::{
    collections::HashMap,
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    num::NonZeroUsize,
    sync::Arc,
};

use bytes::Bytes;
use clap::Parser;
use futures_util::{SinkExt, TryStreamExt};
use log::LevelFilter;
use ordered_float::NotNan;
use shared::{
    deserialize, framed_tcp_stream, serialize, BlockDesc, BlockDescKind, Direction, ElemColor,
    ItemDesc, MapDesc, Message, ModelId,
};
use tokio::{
    net::{TcpListener, TcpStream},
    runtime, select,
    sync::{mpsc, Mutex},
};

#[derive(clap::Parser)]
struct Args {
    #[arg(short, long, default_value_t = std::thread::available_parallelism().expect("failed to obtain available parallelism"))]
    num_threads: NonZeroUsize,
    #[arg(short, long, default_value_t = 8369)]
    port: u16,
}

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let args = Args::parse();

    let num_threads = args.num_threads.get();

    log::info!("number of threads: {num_threads}");

    let mut runtime_builder = if num_threads == 1 {
        runtime::Builder::new_current_thread()
    } else {
        let mut runtime_builder = runtime::Builder::new_multi_thread();
        runtime_builder.worker_threads(num_threads);

        runtime_builder
    };

    let runtime = runtime_builder
        .enable_io()
        .build()
        .expect("failed to create tokio runtime");

    runtime.block_on(async {
        let ip_addr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
        let socket_addr = SocketAddr::new(ip_addr, args.port);

        log::info!("socket address: {socket_addr}");

        let tcp_listener = TcpListener::bind(socket_addr)
            .await
            .expect("failed to create tcp listener");

        let state = Arc::new(Mutex::new(State {
            clients: HashMap::new(),
        }));

        loop {
            match tcp_listener.accept().await {
                Ok((tcp_stream, socket_addr)) => {
                    let state = Arc::clone(&state);

                    runtime.spawn(handle_connection(state, tcp_stream, socket_addr));
                }
                Err(error) => {
                    log::error!("{error}");
                }
            }
        }
    });
}

struct State {
    clients: HashMap<SocketAddr, mpsc::UnboundedSender<Bytes>>,
}

async fn handle_connection(
    state: Arc<Mutex<State>>,
    tcp_stream: TcpStream,
    socket_addr: SocketAddr,
) {
    log::info!("accepted connection to: {socket_addr}");

    let (sender, receiver) = mpsc::unbounded_channel();

    state.lock().await.clients.insert(socket_addr, sender);

    if let Err(error) = handle_client(&state, tcp_stream, receiver).await {
        log::error!("{error}");
    }

    state.lock().await.clients.remove(&socket_addr);

    log::info!("closed connection to: {socket_addr}")
}

async fn handle_client(
    state: &Arc<Mutex<State>>,
    tcp_stream: TcpStream,
    mut receiver: mpsc::UnboundedReceiver<Bytes>,
) -> Result<(), Box<dyn Error>> {
    let mut framed_tcp_stream = framed_tcp_stream(tcp_stream);

    let block_desc = BlockDesc {
        model_id: ModelId::Game {
            name: "RoadTechStraight".to_owned(),
        },
        elem_color: ElemColor::Blue,
        kind: BlockDescKind::Normal {
            x: 20,
            y: 21,
            z: 22,
            direction: Direction::East,
            is_ground: false,
            is_ghost: false,
        },
    };

    let item_desc = ItemDesc {
        model_id: ModelId::Game {
            name: "CactusMedium".to_owned(),
        },
        x: NotNan::new(300.0).unwrap(),
        y: NotNan::new(300.0).unwrap(),
        z: NotNan::new(300.0).unwrap(),
        yaw: NotNan::new(0.0).unwrap(),
        pitch: NotNan::new(0.0).unwrap(),
        roll: NotNan::new(0.0).unwrap(),
    };

    let map_desc = MapDesc {
        blocks: vec![block_desc],
        items: vec![item_desc],
    };

    let frame = serialize(&map_desc)?;

    framed_tcp_stream.send(frame.into()).await?;

    loop {
        select! {
            result = framed_tcp_stream.try_next() => match result? {
                None => break,
                Some(frame) => handle_frame(state, frame.freeze()).await?,
            },
            option = receiver.recv() => match option {
                None => break,
                Some(frame) => framed_tcp_stream.send(frame).await?,
            }
        }
    }

    Ok(())
}

async fn handle_frame(state: &Arc<Mutex<State>>, frame: Bytes) -> Result<(), Box<dyn Error>> {
    let message: Message = deserialize(&frame)?;

    match message {
        Message::PlaceBlock(block_desc) => {
            println!("placed block: {block_desc:?}");
        }
        Message::RemoveBlock(block_desc) => {
            println!("removed block: {block_desc:?}");
        }
        Message::PlaceItem(item_desc) => {
            println!("placed item: {item_desc:?}");
        }
        Message::RemoveItem(item_desc) => {
            println!("removed item {item_desc:?}");
        }
        Message::AddBlockModel { .. } => {}
        Message::AddItemModel { .. } => {}
    }

    Ok(())
}
