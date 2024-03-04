use std::{
    error::Error,
    ffi::{c_char, CStr, CString},
    net::{IpAddr, SocketAddr},
    ptr::null,
    str::FromStr,
    sync::Mutex,
    thread,
};

use futures_util::TryStreamExt;
use serde::Deserialize;
use tokio::{net::TcpStream, runtime, select};
use tokio_util::{codec::LengthDelimitedCodec, sync::CancellationToken};

static STATE: Mutex<State> = Mutex::new(State::Disconnected);
static STATUS_TEXT: Mutex<Option<CString>> = Mutex::new(None);

#[no_mangle]
extern "C" fn ButtonLabel() -> *const c_char {
    match *STATE.lock().unwrap() {
        State::Disconnected => "Join\0".as_ptr() as *const c_char,
        State::Joining { .. } => "Cancel\0".as_ptr() as *const c_char,
    }
}

#[no_mangle]
extern "C" fn ButtonPressed(host: *const c_char, port: *const c_char) {
    let mut state = STATE.lock().unwrap();

    match *state {
        State::Disconnected => {
            let cancellation_token = CancellationToken::new();
            let child_token = cancellation_token.child_token();

            *state = State::Joining { cancellation_token };
            *STATUS_TEXT.lock().unwrap() = Some(CString::new("connecting...").unwrap());

            let host = unsafe { CStr::from_ptr(host).to_str().unwrap().to_owned() };
            let port = unsafe { u16::from_str(CStr::from_ptr(port).to_str().unwrap()).unwrap() };

            thread::spawn(move || {
                if let Err(error) = main(child_token, host.as_str(), port) {
                    *STATE.lock().unwrap() = State::Disconnected;
                    *STATUS_TEXT.lock().unwrap() = Some(CString::new(error.to_string()).unwrap());
                }
            });
        }
        State::Joining {
            ref cancellation_token,
        } => {
            cancellation_token.cancel();

            *state = State::Disconnected;
            *STATUS_TEXT.lock().unwrap() = Some(CString::new("canceled").unwrap());
        }
    }
}

#[no_mangle]
extern "C" fn StatusText() -> *const c_char {
    match &*STATUS_TEXT.lock().unwrap() {
        None => null(),
        Some(status_text) => status_text.as_ptr(),
    }
}

fn main(
    cancellation_token: CancellationToken,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn Error>> {
    let runtime = runtime::Builder::new_current_thread().enable_io().build()?;

    runtime.block_on(async {
        let ip_addr = IpAddr::from_str(host)?;
        let socket_addr = SocketAddr::new(ip_addr, port);

        let tcp_stream = select! {
            _ = cancellation_token.cancelled() => return Ok(()),
            tcp_stream = TcpStream::connect(socket_addr) => tcp_stream?,
        };

        *STATUS_TEXT.lock().unwrap() = Some(CString::new("connected").unwrap());

        let mut framed_tcp_stream = LengthDelimitedCodec::builder().new_framed(tcp_stream);

        let frame = select! {
            _ = cancellation_token.cancelled() => return Ok(()),
            frame = framed_tcp_stream.try_next() => frame?.ok_or("connection lost")?.freeze(),
        };

        let map: Map = postcard::from_bytes(&frame)?;

        for block in map.blocks {}

        for item in map.items {}

        Ok(())
    })
}

enum State {
    Disconnected,
    Joining {
        cancellation_token: CancellationToken,
    },
}

#[derive(Deserialize)]
struct Map {
    blocks: Vec<()>,
    items: Vec<()>,
}
