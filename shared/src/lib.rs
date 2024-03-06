use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub fn framed<T: AsyncRead + AsyncWrite>(stream: T) -> Framed<T, LengthDelimitedCodec> {
    LengthDelimitedCodec::builder().new_framed(stream)
}
