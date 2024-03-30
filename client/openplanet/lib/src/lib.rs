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
use futures::{executor::block_on, task::noop_waker_ref, SinkExt, TryStreamExt};
use game::{hook_place_block, hook_place_item, hook_remove_block, hook_remove_item};
use native_dialog::{MessageDialog, MessageType};
use shared::{
    deserialize, framed_tcp_stream, serialize, Block, Direction, ElemColor, FramedTcpStream,
    FreeBlock, Item, Message,
};
use tokio::{net::TcpStream, select};
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
        .expect("No open connection");

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
    context.framed_tcp_stream = None;
}

// context //

#[repr(C)]
struct Context {
    state: State,
    status_text_buf: Box<[u8; 256]>,
    map_editor: Option<NonZeroUsize>,

    connection_future: Option<ConnectionFuture>,
    framed_tcp_stream: Option<FramedTcpStream>,
}

impl Context {
    fn new() -> Self {
        Self {
            state: State::Disconnected,
            status_text_buf: Box::new([0; 256]),
            map_editor: None,
            connection_future: None,
            framed_tcp_stream: None,
        }
    }

    fn set_status_text(&mut self, status_text: &str) {
        if status_text.len() >= self.status_text_buf.len() {
            panic!("Status text is too long for buffer");
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

    context.framed_tcp_stream = Some(framed_tcp_stream(tcp_stream));

    let user_data = context as *mut Context as *mut u8;
    let _place_block_hook = hook_place_block(user_data, place_block_callback)?;
    let _remove_block_hook = hook_remove_block(user_data, remove_block_callback)?;
    let _place_item_hook = hook_place_item(user_data, place_item_callback)?;
    let _remove_item_hook = hook_remove_item(user_data, remove_item_callback)?;

    loop {
        select! {
            result = context.framed_tcp_stream.as_mut().unwrap().try_next() => match result? {
                None => return Err("Server closed connection".into()),
                Some(frame) => {
                    let message = deserialize(&frame)?;

                    match message {
                        Message::PlaceBlock(..) => {}
                        Message::RemoveBlock(..) => {}
                        Message::PlaceFreeBlock(..) => {}
                        Message::RemoveFreeBlock(..)=> {}
                        Message::PlaceItem(..) => {}
                        Message::RemoveItem(..) => {}
                    }
                }
            }
        }
    }
}

// hook callbacks //

unsafe extern "system" fn place_block_callback(user_data: *mut u8, block: *mut game::Block) {
    let slice = std::slice::from_raw_parts(block as *mut u8, 444);

    let context = &mut *(user_data as *mut Context);
    let block = &*block;

    let _ = MessageDialog::new()
        .set_type(MessageType::Error)
        .set_title(FILE_NAME)
        .set_text(&format!("{:02X?}", slice))
        .show_alert();

    let direction = match block.direction {
        0 => Direction::North,
        1 => Direction::East,
        2 => Direction::South,
        3 => Direction::West,
        _ => unreachable!(),
    };

    let is_ground = block.flags & 0x00001000 != 0;

    let is_ghost = block.flags & 0x10000000 != 0;

    let is_free = block.flags & 0x20000000 != 0;

    let elem_color = ElemColor::Default;

    let message = if is_free {
        Message::PlaceFreeBlock(FreeBlock {
            x: block.x_pos,
            y: block.y_pos,
            z: block.z_pos,
            yaw: block.yaw,
            pitch: block.pitch,
            roll: block.roll,
            elem_color,
        })
    } else {
        Message::PlaceBlock(Block {
            x: block.x_coord as u8,
            y: block.y_coord as u8,
            z: block.z_coord as u8,
            direction,
            is_ground,
            is_ghost,
            elem_color,
        })
    };

    send_message(context, &message);
}

unsafe extern "system" fn remove_block_callback(user_data: *mut u8, block: *mut game::Block) {
    let context = &mut *(user_data as *mut Context);
    let block = &*block;

    let direction = match block.direction {
        0 => Direction::North,
        1 => Direction::East,
        2 => Direction::South,
        3 => Direction::West,
        _ => unreachable!(),
    };

    let is_ground = block.flags & 0x00001000 != 0;

    let is_ghost = block.flags & 0x10000000 != 0;

    let is_free = block.flags & 0x20000000 != 0;

    let elem_color = ElemColor::Default;

    let message = if is_free {
        Message::RemoveFreeBlock(FreeBlock {
            x: block.x_pos,
            y: block.y_pos,
            z: block.z_pos,
            yaw: block.yaw,
            pitch: block.pitch,
            roll: block.roll,
            elem_color,
        })
    } else {
        Message::RemoveBlock(Block {
            x: block.x_coord as u8,
            y: block.y_coord as u8,
            z: block.z_coord as u8,
            direction,
            is_ground,
            is_ghost,
            elem_color,
        })
    };

    send_message(context, &message);
}

unsafe extern "system" fn place_item_callback(
    user_data: *mut u8,
    _item_params: *mut game::ItemParams,
) {
    let context = &mut *(user_data as *mut Context);

    let message = Message::PlaceItem(Item);

    send_message(context, &message);
}

unsafe extern "system" fn remove_item_callback(user_data: *mut u8, _item: *mut game::Item) {
    let context = &mut *(user_data as *mut Context);

    let message = Message::RemoveItem(Item);

    send_message(context, &message);
}

fn send_message(context: &mut Context, message: &Message) {
    let frame = serialize(&message).expect("Failed to serialize message");

    block_on(async {
        context
            .framed_tcp_stream
            .as_mut()
            .expect("TCP stream not initialized")
            .send(frame.into())
            .await
            .expect("Failed to send frame");
    });
}

// utils //

unsafe fn convert_c_string(c_string: *const c_char) -> String {
    CStr::from_ptr(c_string)
        .to_str()
        .expect("Invalid UTF-8 string")
        .to_owned()
}
