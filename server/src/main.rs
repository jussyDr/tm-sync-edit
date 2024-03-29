use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    num::NonZeroUsize,
};

use clap::Parser;
use futures_util::TryStreamExt;
use log::LevelFilter;
use tokio::{
    net::{TcpListener, TcpStream},
    runtime, select,
};
use tokio_util::codec::{Decoder, LengthDelimitedCodec};

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

        loop {
            match tcp_listener.accept().await {
                Ok((tcp_stream, socket_addr)) => {
                    runtime.spawn(handle_connection(tcp_stream, socket_addr));
                }
                Err(error) => {
                    log::error!("{error}");
                }
            }
        }
    });
}

async fn handle_connection(tcp_stream: TcpStream, socket_addr: SocketAddr) {
    log::info!("accepted connection to: {socket_addr}");

    match handle_client(tcp_stream).await {
        Ok(()) => log::info!("closed connection to: {socket_addr}"),
        Err(error) => log::error!("{error}"),
    }
}

async fn handle_client(tcp_stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut framed_tcp_stream = LengthDelimitedCodec::new().framed(tcp_stream);

    loop {
        select! {
            result = framed_tcp_stream.try_next() => match result? {
                None => break,
                Some(_frame) => {}
            }
        }
    }

    Ok(())
}
