use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Framed, LengthDelimitedCodec};

pub fn framed_tcp_stream(tcp_stream: TcpStream) -> Framed<TcpStream, LengthDelimitedCodec> {
    LengthDelimitedCodec::new().framed(tcp_stream)
}
