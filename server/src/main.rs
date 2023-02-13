use std::io::Result;
use tm_sync_edit_server::Server;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    let level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };

    tracing_subscriber::fmt().with_max_level(level).init();

    Server::run().await
}
