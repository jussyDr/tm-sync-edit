use std::{
    ffi::{c_char, CStr},
    net::{IpAddr, SocketAddr},
    pin::Pin,
    str::FromStr,
    task::Poll,
};

use futures::{poll, Future, TryStreamExt};
use shared::framed_tcp_stream;
use tokio::{
    net::TcpStream,
    runtime::{self, Runtime},
    select,
};

struct Context {
    runtime: Runtime,
    main_future: Pin<Box<dyn Future<Output = ()>>>,
}

#[no_mangle]
unsafe extern "system" fn Join(host: *const c_char, port: *const c_char) -> *mut Context {
    let host = CStr::from_ptr(host)
        .to_str()
        .expect("invalid UTF-8")
        .to_owned();

    let port = CStr::from_ptr(port)
        .to_str()
        .expect("invalid UTF-8")
        .to_owned();

    let runtime = runtime::Builder::new_current_thread()
        .build()
        .expect("failed to create tokio runtime");

    let main_future = Box::pin(main(host, port));

    let context = Context {
        runtime,
        main_future,
    };

    Box::into_raw(Box::new(context))
}

#[no_mangle]
unsafe extern "system" fn Update(context: *mut Context) {
    let context = &mut *context;

    context
        .runtime
        .block_on(async { if let Poll::Ready(_value) = poll!(&mut context.main_future) {} })
}

async fn main(host: String, port: String) {
    let ip_addr = IpAddr::from_str(&host).expect("invalid IP address");
    let port = u16::from_str(&port).expect("invalid port");
    let socket_addr = SocketAddr::new(ip_addr, port);

    let tcp_stream = TcpStream::connect(socket_addr)
        .await
        .expect("failed to connect");

    let mut framed_tcp_stream = framed_tcp_stream(tcp_stream);

    loop {
        select! {
            result = framed_tcp_stream.try_next() => {
                let frame = result.expect("error while receiving tcp stream frame").expect("server disconnected");
            }
        }
    }
}
