mod game;
mod hook;
mod windows;

use std::{
    error::Error,
    ffi::{c_char, c_void, CStr},
    io,
    net::{IpAddr, SocketAddr, TcpStream},
    panic,
    str::FromStr,
    sync::Mutex,
};

use windows::{message_box, DllCallReason, MessageBoxType};
use windows_sys::Win32::Foundation::{BOOL, HINSTANCE, TRUE};

static STATE: Mutex<State> = Mutex::new(State::new());

struct State {
    tcp_stream: Option<TcpStream>,
}

impl State {
    const fn new() -> Self {
        Self { tcp_stream: None }
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

fn update() -> Result<(), io::Error> {
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

    let tcp_stream = TcpStream::connect(socket_addr)?;

    STATE.lock().unwrap().tcp_stream = Some(tcp_stream);

    Ok(())
}
