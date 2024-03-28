use std::{
    error::Error,
    ffi::{c_char, CStr},
    future::Future,
    net::{IpAddr, SocketAddr},
    panic,
    pin::Pin,
    str::FromStr,
    task::Poll,
};

use futures::{poll, TryStreamExt};
use native_dialog::{MessageDialog, MessageType};
use shared::framed_tcp_stream;
use tokio::{
    net::TcpStream,
    runtime::{self, Runtime},
    select,
};

#[no_mangle]
unsafe extern "system" fn Join(host: *const c_char, port: *const c_char) -> *mut Main {
    panic::set_hook(Box::new(|panic_info| {
        let _ = MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("SyncEdit.dll")
            .set_text(&panic_info.to_string())
            .show_alert();
    }));

    let host = CStr::from_ptr(host)
        .to_str()
        .expect("null byte in host string")
        .to_owned();

    let port = CStr::from_ptr(port)
        .to_str()
        .expect("null byte in port string")
        .to_owned();

    let runtime = runtime::Builder::new_current_thread()
        .build()
        .expect("failed to create tokio runtime");

    let future = Box::pin(main(host, port));

    let main = Main { runtime, future };

    Box::into_raw(Box::new(main))
}

#[no_mangle]
unsafe extern "system" fn Update(main: *mut Main) {
    let main = &mut *main;

    main.runtime.block_on(async {
        if let Poll::Ready(Err(error)) = poll!(&mut main.future) {
            panic!("{error}");
        }
    });
}

#[no_mangle]
unsafe extern "system" fn Destroy(main: *mut Main) {
    drop(Box::from_raw(main));
}

type MainFuture = Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>>>>;

struct Main {
    runtime: Runtime,
    future: MainFuture,
}

async fn main(host: String, port: String) -> Result<(), Box<dyn Error>> {
    let ip_addr = IpAddr::from_str(&host)?;
    let port = u16::from_str(&port)?;
    let socket_addr = SocketAddr::new(ip_addr, port);

    let tcp_stream = TcpStream::connect(socket_addr).await?;

    let mut framed_tcp_stream = framed_tcp_stream(tcp_stream);

    loop {
        select! {
            result = framed_tcp_stream.try_next() => match result? {
                None => return Err("server closed".into()),
                Some(_frame) => {}
            }
        }
    }
}
