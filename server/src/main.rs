use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8369").await.unwrap();

    loop {
        let (_stream, _addr) = listener.accept().await.unwrap();

        tokio::spawn(async {});
    }
}
