mod game;

use std::{
    error::Error,
    ffi::{c_char, c_void, CStr},
    future::{poll_fn, Future},
    net::{IpAddr, SocketAddr},
    num::NonZeroUsize,
    panic,
    pin::Pin,
    str::FromStr,
    task::{self, Poll},
};

use async_compat::CompatExt;
use futures::{task::noop_waker_ref, TryStreamExt};
use native_dialog::{MessageDialog, MessageType};
use tokio::{net::TcpStream, select};
use tokio_util::codec::{Decoder, LengthDelimitedCodec};
use windows_sys::Win32::{
    Foundation::{BOOL, HINSTANCE, TRUE},
    System::SystemServices::DLL_PROCESS_ATTACH,
};

// main //

const FILE_NAME: &str = "SyncEdit.dll";

#[no_mangle]
unsafe extern "system" fn DllMain(
    _module: HINSTANCE,
    call_reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    if call_reason == DLL_PROCESS_ATTACH {
        panic::set_hook(Box::new(|panic_info| {
            let _ = MessageDialog::new()
                .set_type(MessageType::Error)
                .set_title(FILE_NAME)
                .set_text(&panic_info.to_string())
                .show_alert();
        }));
    }

    TRUE
}

// api //

#[no_mangle]
unsafe extern "system" fn CreateContext() -> *mut Context {
    let mut context = Context::new();
    context.set_status_text("Disconnected");

    Box::into_raw(Box::new(context))
}

#[no_mangle]
unsafe extern "system" fn DestroyContext(context: *mut Context) {
    drop(Box::from_raw(context));
}

#[no_mangle]
unsafe extern "system" fn OpenConnection(
    context: *mut Context,
    host: *const c_char,
    port: *const c_char,
) {
    (*context).state = State::Connecting;

    let host = convert_c_string(host);
    let port = convert_c_string(port);

    let connection_future = Box::pin(connection(&mut *context, host, port));

    (*context).connection_future = Some(connection_future);
}

#[no_mangle]
unsafe extern "system" fn UpdateConnection(context: *mut Context) {
    let context = &mut *context;

    let connection_future = context
        .connection_future
        .as_mut()
        .expect("no open connection");

    let mut task_context = task::Context::from_waker(noop_waker_ref());

    if let Poll::Ready(Err(error)) = connection_future.as_mut().poll(&mut task_context) {
        context.state = State::Disconnected;
        context.set_status_text(&error.to_string());
    }
}

#[no_mangle]
unsafe extern "system" fn CloseConnection(context: *mut Context) {
    let context = &mut *context;

    context.state = State::Disconnected;
    context.connection_future = None;
}

// context //

#[repr(C)]
struct Context {
    state: State,
    status_text_buf: Box<[u8; 256]>,
    map_editor: Option<NonZeroUsize>,

    connection_future: Option<ConnectionFuture>,
}

impl Context {
    fn new() -> Self {
        Self {
            state: State::Disconnected,
            status_text_buf: Box::new([0; 256]),
            map_editor: None,
            connection_future: None,
        }
    }

    fn set_status_text(&mut self, status_text: &str) {
        if status_text.len() >= self.status_text_buf.len() {
            panic!("status text is too long for buffer");
        }

        self.status_text_buf[..status_text.len()].copy_from_slice(status_text.as_bytes());
        self.status_text_buf[status_text.len()] = 0;
    }
}

#[repr(u8)]
enum State {
    Disconnected,
    Connecting,
    OpeningMapEditor,
    Connected,
}

// connection //

type ConnectionFuture = Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>>>>;

async fn connection(
    context: &mut Context,
    host: String,
    port: String,
) -> Result<(), Box<dyn Error>> {
    let ip_addr = IpAddr::from_str(&host)?;
    let port = u16::from_str(&port)?;
    let socket_addr = SocketAddr::new(ip_addr, port);

    context.set_status_text("Connecting...");

    let tcp_stream = TcpStream::connect(socket_addr).compat().await?;

    context.map_editor = None;

    context.state = State::OpeningMapEditor;
    context.set_status_text("Opening map editor...");

    poll_fn(|_cx| {
        if context.map_editor.is_some() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    })
    .await;

    context.state = State::Connected;
    context.set_status_text("Connected");

    let mut framed_tcp_stream = LengthDelimitedCodec::new().framed(tcp_stream);

    loop {
        select! {
            result = framed_tcp_stream.try_next() => match result? {
                None => return Err("server closed connection".into()),
                Some(_frame) => {}
            }
        }
    }
}

// utils //

unsafe fn convert_c_string(c_string: *const c_char) -> String {
    CStr::from_ptr(c_string)
        .to_str()
        .expect("invalid UTF-8 string")
        .to_owned()
}
