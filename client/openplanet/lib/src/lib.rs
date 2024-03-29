mod game;

use std::{
    error::Error,
    ffi::{c_char, c_void, CStr},
    future::Future,
    net::{IpAddr, SocketAddr},
    panic,
    pin::Pin,
    str::FromStr,
    task,
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
    Box::into_raw(Box::new(Context::new()))
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
    let host = convert_c_string(host);
    let port = convert_c_string(port);

    let connection_future = Box::pin(connection(host, port));

    (*context).connection_future = Some(connection_future);
}

#[no_mangle]
unsafe extern "system" fn UpdateConnection(context: *mut Context) -> bool {
    let connection_future = (*context)
        .connection_future
        .as_mut()
        .expect("no open connection");

    let mut task_context = task::Context::from_waker(noop_waker_ref());

    connection_future
        .as_mut()
        .poll(&mut task_context)
        .is_pending()
}

#[no_mangle]
unsafe extern "system" fn CloseConnection(context: *mut Context) {
    (*context).connection_future = None;
}

// context //

struct Context {
    connection_future: Option<ConnectionFuture>,
}

impl Context {
    fn new() -> Self {
        Self {
            connection_future: None,
        }
    }
}

// connection //

type ConnectionFuture = Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>>>>;

async fn connection(host: String, port: String) -> Result<(), Box<dyn Error>> {
    let ip_addr = IpAddr::from_str(&host)?;
    let port = u16::from_str(&port)?;
    let socket_addr = SocketAddr::new(ip_addr, port);

    let tcp_stream = TcpStream::connect(socket_addr).compat().await?;

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
