use std::{
    error::Error,
    ffi::{c_char, CStr, CString},
    net::{IpAddr, SocketAddr, TcpStream},
    ops::Deref,
    ptr::null,
    str::FromStr,
    sync::Mutex,
    time::Duration,
};

static LAST_ERROR_STRING: Mutex<Option<CString>> = Mutex::new(None);

#[no_mangle]
extern "C" fn LastErrorString() -> *const c_char {
    match LAST_ERROR_STRING.lock().unwrap().deref() {
        None => null(),
        Some(last_error_string) => last_error_string.as_ptr(),
    }
}

#[no_mangle]
extern "C" fn Join(host: *const c_char, port: u16) -> bool {
    if let Err(error) = join(host, port) {
        let error_string = unsafe { CString::from_vec_unchecked(error.to_string().into_bytes()) };
        *LAST_ERROR_STRING.lock().unwrap() = Some(error_string);

        false
    } else {
        true
    }
}

fn join(host: *const c_char, port: u16) -> Result<(), Box<dyn Error>> {
    let host = unsafe { CStr::from_ptr(host) }.to_str()?;

    let ip_addr = IpAddr::from_str(host)?;
    let socket_addr = SocketAddr::new(ip_addr, port);

    let _tcp_stream = TcpStream::connect_timeout(&socket_addr, Duration::from_millis(200))?;

    Ok(())
}
