mod game;
mod hook;
mod windows;

use std::{
    error::Error,
    ffi::{c_char, c_void, CStr},
    future::Future,
    io,
    net::{IpAddr, SocketAddr},
    panic,
    pin::Pin,
    str::FromStr,
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake, Waker},
};

use futures::{TryFuture, TryStream};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use windows::{message_box, DllCallReason, MessageBoxType};
use windows_sys::Win32::Foundation::{BOOL, HINSTANCE, TRUE};

static STATE: Mutex<State> = Mutex::new(State::new());

enum State {
    Disconnected,
    Connecting {
        tcp_stream_connect: Pin<Box<dyn Future<Output = io::Result<TcpStream>> + Send>>,
        waker: Arc<TcpStreamConnectWaker>,
    },
    Connected {
        framed_tcp_stream: Framed<TcpStream, LengthDelimitedCodec>,
        waker: Arc<TcpStreamWaker>,
    },
}

impl State {
    const fn new() -> Self {
        Self::Disconnected
    }
}

#[no_mangle]
extern "system" fn DllMain(
    _dll_module: HINSTANCE,
    call_reason: DllCallReason,
    _reserved: *mut c_void,
) -> BOOL {
    match call_reason {
        DllCallReason::ProcessAttach => {
            panic::set_hook(Box::new(|panic_info| {
                let text = panic_info.to_string();
                let caption = "SyncEdit.dll";

                message_box(&text, caption, MessageBoxType::Error).unwrap();
            }));
        }
        DllCallReason::ProcessDettach => {
            let _ = panic::take_hook();
        }
        _ => {}
    }

    TRUE
}

#[no_mangle]
extern "C" fn Init() {
    init().unwrap();
}

fn init() -> Result<(), ()> {
    Ok(())
}

#[no_mangle]
extern "C" fn Update() {
    update().unwrap();
}

fn update() -> Result<(), Box<dyn Error>> {
    let mut state = STATE.lock().unwrap();

    match &mut *state {
        State::Disconnected => {}
        State::Connecting {
            tcp_stream_connect,
            waker,
        } => {
            let waker = Waker::from(Arc::clone(waker));
            let mut context = Context::from_waker(&waker);

            if let Poll::Ready(tcp_stream) = tcp_stream_connect.as_mut().try_poll(&mut context) {
                let framed_tcp_stream = LengthDelimitedCodec::builder().new_framed(tcp_stream?);
                let waker = Arc::new(TcpStreamWaker::new());

                *state = State::Connected {
                    framed_tcp_stream,
                    waker,
                };
            }
        }
        State::Connected {
            framed_tcp_stream,
            waker,
        } => {
            let mut framed_tcp_stream = Pin::new(framed_tcp_stream);

            let waker = Waker::from(Arc::clone(waker));
            let mut context = Context::from_waker(&waker);

            loop {
                match framed_tcp_stream.as_mut().try_poll_next(&mut context) {
                    Poll::Pending => break,
                    Poll::Ready(None) => {
                        todo!("disconnected")
                    }
                    Poll::Ready(Some(frame)) => {
                        let frame = frame?.freeze();

                        let message = postcard::from_bytes(&frame)?;

                        match message {
                            Message::PlaceBlock => {}
                            Message::RemoveBlock => {}
                            Message::PlaceItem => {}
                            Message::RemoveItem => {}
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[no_mangle]
extern "C" fn Destroy() {
    destroy().unwrap();
}

fn destroy() -> Result<(), ()> {
    Ok(())
}

#[no_mangle]
extern "C" fn Join(host: *const c_char, port: u16) {
    join(host, port).unwrap()
}

fn join(host: *const c_char, port: u16) -> Result<(), Box<dyn Error>> {
    let host = unsafe { CStr::from_ptr(host) }.to_str()?;

    let ip_addr = IpAddr::from_str(host)?;
    let socket_addr = SocketAddr::new(ip_addr, port);

    let tcp_stream_connect = Box::pin(TcpStream::connect(socket_addr));
    let waker = Arc::new(TcpStreamConnectWaker::new());

    *STATE.lock().unwrap() = State::Connecting {
        tcp_stream_connect,
        waker,
    };

    Ok(())
}

struct TcpStreamConnectWaker;

impl TcpStreamConnectWaker {
    fn new() -> Self {
        Self
    }
}

impl Wake for TcpStreamConnectWaker {
    fn wake(self: Arc<Self>) {}

    fn wake_by_ref(self: &Arc<Self>) {}
}

struct TcpStreamWaker;

impl TcpStreamWaker {
    fn new() -> Self {
        Self
    }
}

impl Wake for TcpStreamWaker {
    fn wake(self: Arc<Self>) {}

    fn wake_by_ref(self: &Arc<Self>) {}
}

#[derive(Serialize, Deserialize)]
enum Message {
    PlaceBlock,
    RemoveBlock,
    PlaceItem,
    RemoveItem,
}
