use std::{
    collections::HashMap,
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    thread,
};

use bytes::Bytes;
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use log::LevelFilter;
use tokio::{
    net::{TcpListener, TcpStream},
    runtime, select, spawn,
    sync::mpsc::{self},
};
use tokio_util::codec::LengthDelimitedCodec;

#[derive(clap::Parser)]
struct Args {
    #[arg(short, long)]
    port: Option<u16>,
    #[arg(short, long)]
    num_threads: Option<NonZeroUsize>,
}

fn main() -> io::Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let args = Args::parse();

    let num_threads = args.num_threads.unwrap_or(thread::available_parallelism()?);

    log::info!("number of worker threads: {num_threads}");

    let mut runtime_builder = if num_threads.get() == 1 {
        runtime::Builder::new_current_thread()
    } else {
        let mut runtime_builder = runtime::Builder::new_multi_thread();
        runtime_builder.worker_threads(num_threads.get());
        runtime_builder
    };

    let runtime = runtime_builder.enable_io().build()?;

    runtime.block_on(async {
        let state = Arc::new(Mutex::new(State::new()));

        let port = args.port.unwrap_or(8369);

        let socket_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);

        let tcp_listener = TcpListener::bind(socket_addr).await?;

        log::info!("listening on: {socket_addr}");

        loop {
            let (tcp_stream, socket_addr) = tcp_listener.accept().await?;

            let state = Arc::clone(&state);

            spawn(handle_connection(state, tcp_stream, socket_addr));
        }
    })
}

async fn handle_connection(
    state: Arc<Mutex<State>>,
    tcp_stream: TcpStream,
    socket_addr: SocketAddr,
) {
    let (sender, receiver) = mpsc::unbounded_channel();

    {
        let mut state = state.lock().unwrap();
        state.clients.insert(socket_addr, sender);
    }

    if let Err(err) = handle_client(&state, tcp_stream, receiver).await {
        log::error!("{err}");
    }

    {
        let mut state = state.lock().unwrap();
        state.clients.remove(&socket_addr);
    }
}

async fn handle_client(
    _state: &Arc<Mutex<State>>,
    tcp_stream: TcpStream,
    mut receiver: mpsc::UnboundedReceiver<Bytes>,
) -> io::Result<()> {
    let mut framed_tcp_stream = LengthDelimitedCodec::builder().new_framed(tcp_stream);

    loop {
        select! {
            Some(frame) = receiver.recv() => {
                framed_tcp_stream.send(frame).await?;
            },
            Some(_frame) = framed_tcp_stream.next() => {},
            else => break
        }
    }

    Ok(())
}

struct State {
    clients: HashMap<SocketAddr, mpsc::UnboundedSender<Bytes>>,
}

impl State {
    fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }
}
