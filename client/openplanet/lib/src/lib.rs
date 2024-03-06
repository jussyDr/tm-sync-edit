use std::{
    collections::HashMap,
    error::Error,
    ffi::{c_char, CStr, CString},
    net::{IpAddr, SocketAddr},
    ptr::null,
    str::FromStr,
    sync::Mutex,
    thread,
};

use futures_util::TryStreamExt;
use once_cell::sync::Lazy;
use serde::Deserialize;
use tokio::{net::TcpStream, runtime, select};
use tokio_util::{codec::LengthDelimitedCodec, sync::CancellationToken};

static STATE: Mutex<State> = Mutex::new(State::Disconnected);

static STATUS_TEXT: Mutex<Option<CString>> = Mutex::new(None);

static EDITOR: Mutex<usize> = Mutex::new(0);

static BLOCK_INFOS: Lazy<Mutex<HashMap<String, usize>>> = Lazy::new(|| Mutex::new(HashMap::new()));

static ITEM_MODELS: Lazy<Mutex<HashMap<String, usize>>> = Lazy::new(|| Mutex::new(HashMap::new()));

static SHOULD_OPEN_EDITOR: Mutex<bool> = Mutex::new(false);

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
                if let Err(error) = connection(child_token, host.as_str(), port) {
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

#[no_mangle]
extern "C" fn SetEditor(editor: usize) {
    *EDITOR.lock().unwrap() = editor;
}

#[no_mangle]
extern "C" fn ShouldOpenEditor() -> bool {
    let mut should_open_editor = SHOULD_OPEN_EDITOR.lock().unwrap();

    if *should_open_editor {
        *should_open_editor = false;

        true
    } else {
        false
    }
}

#[no_mangle]
extern "C" fn RegisterBlockInfo(id: *const c_char, block_info: usize) {
    let id = unsafe { CStr::from_ptr(id).to_str().unwrap().to_owned() };

    BLOCK_INFOS.lock().unwrap().insert(id, block_info);
}

#[no_mangle]
extern "C" fn RegisterItemModel(id: *const c_char, item_model: usize) {
    let id = unsafe { CStr::from_ptr(id).to_str().unwrap().to_owned() };

    ITEM_MODELS.lock().unwrap().insert(id, item_model);
}

fn connection(
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

        let game_fns = GameFns::find()?;

        let frame = select! {
            _ = cancellation_token.cancelled() => return Ok(()),
            frame = framed_tcp_stream.try_next() => frame?.ok_or("connection lost")?.freeze(),
        };

        let map: Map = postcard::from_bytes(&frame)?;

        for block in map.blocks {
            let block_info = *BLOCK_INFOS.lock().unwrap().get(&block.id).ok_or("")?;
        }

        for free_block in map.free_blocks {
            let block_info = *BLOCK_INFOS.lock().unwrap().get(&free_block.id).ok_or("")?;
        }

        for item in map.items {
            let item_model = *ITEM_MODELS.lock().unwrap().get(&item.id).ok_or("")?;
        }

        Ok(())
    })
}

enum State {
    Disconnected,
    Joining {
        cancellation_token: CancellationToken,
    },
}

struct GameFns;

impl GameFns {
    fn find() -> Result<Self, Box<dyn Error>> {
        Ok(Self)
    }

    fn place_block(
        &self,
        editor: usize,
        block_info: usize,
        dir: u32,
        x: u32,
        y: u32,
        z: u32,
        elem_color: u8,
    ) {
        todo!()
    }
}

#[derive(Deserialize)]
struct Map {
    blocks: Vec<Block>,
    free_blocks: Vec<FreeBlock>,
    items: Vec<Item>,
}

#[derive(Deserialize)]
struct Block {
    id: String,
    dir: Direction,
    x: u8,
    y: u8,
    z: u8,
    elem_color: ElemColor,
}

#[derive(Deserialize)]
struct FreeBlock {
    id: String,
}

#[derive(Deserialize)]
struct Item {
    id: String,
}

#[derive(Deserialize)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Deserialize)]
enum ElemColor {
    Default,
    White,
    Green,
    Blue,
    Red,
    Black,
}
