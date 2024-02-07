use std::{
    collections::HashMap,
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    runtime, select, spawn,
    sync::mpsc::{self},
};
use tokio_util::codec::LengthDelimitedCodec;

fn main() -> io::Result<()> {
    let port = 8369;
    let multi_thread = true;

    let mut runtime_builder = if multi_thread {
        runtime::Builder::new_multi_thread()
    } else {
        runtime::Builder::new_current_thread()
    };

    let runtime = runtime_builder.enable_io().build()?;

    runtime.block_on(async {
        let state = Arc::new(Mutex::new(State::new()));

        let socket_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);

        let tcp_listener = TcpListener::bind(socket_addr).await?;

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

    let _ = handle_client(&state, tcp_stream, receiver).await;

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
