use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use log::LevelFilter;
use tokio::{net::TcpListener, runtime, spawn};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .try_init()?;

    let runtime = runtime::Builder::new_multi_thread().enable_io().build()?;

    runtime.block_on(async {
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8369);

        let tcp_listerner = TcpListener::bind(&socket_addr).await?;

        log::info!("listening on {socket_addr}");

        loop {
            let (tcp_stream, socket_addr) = tcp_listerner.accept().await?;

            spawn(async move {
                log::info!("accepted connection to {socket_addr}");
            });
        }
    })
}
