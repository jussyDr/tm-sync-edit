use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use bytes::Bytes;
use futures_util::{SinkExt, TryStreamExt};
use log::LevelFilter;
use tm_sync_edit_shared::framed;
use tokio::{
    net::{TcpListener, TcpStream},
    runtime, select, spawn,
};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let runtime = runtime::Builder::new_multi_thread().enable_io().build()?;

    runtime.block_on(async {
        let ip_addr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
        let port = 8369;
        let socket_addr = SocketAddr::new(ip_addr, port);

        let tcp_listener = TcpListener::bind(socket_addr).await?;

        loop {
            let (tcp_stream, socket_addr) = tcp_listener.accept().await?;

            spawn(async move {
                log::info!("Connection from: {socket_addr}");

                if let Err(error) = handle_client(tcp_stream).await {
                    log::error!("{error}")
                }
            });
        }
    })
}

async fn handle_client(tcp_stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut framed_tcp_stream = framed(tcp_stream);

    framed_tcp_stream.send(Bytes::new()).await?;

    framed_tcp_stream.send(Bytes::new()).await?;

    loop {
        select! {
            result = framed_tcp_stream.try_next() => match result? {
                None => break,
                Some(frame) => {
                    let frame = frame.freeze();
                }
            }
        }
    }

    Ok(())
}
