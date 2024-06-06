mod process;

mod game {
    mod classes;
    mod fns;

    pub use classes::*;
    pub use fns::*;
}

use std::{
    error::Error,
    future::{poll_fn, Future},
    panic,
    pin::Pin,
    task::Poll,
};

use async_compat::CompatExt;
use futures::{executor::block_on, poll, TryStreamExt};
use game::{BackToMainMenuFn, EditNewMap2Fn, EditorCommon, ManiaPlanet, Menus, NodRef};
use process::Process;
use shared::{deserialize, framed_tcp_stream, FramedTcpStream, MapDesc, MapParamsDesc, Mood};
use tokio::net::TcpStream;

#[no_mangle]
extern "system" fn Init(mania_planet: &'static mut ManiaPlanet) -> *mut Context {
    panic::set_hook(Box::new(|panic_info| {
        let _ = native_dialog::MessageDialog::new()
            .set_type(native_dialog::MessageType::Error)
            .set_title("SyncEdit.dll")
            .set_text(&panic_info.to_string())
            .show_alert();
    }));

    let context = Context::new(mania_planet);

    Box::into_raw(Box::new(context))
}

#[no_mangle]
unsafe extern "system" fn Destroy(context: *mut Context) {
    drop(Box::from_raw(context));
}

#[no_mangle]
extern "system" fn Update(context: &mut Context) {
    block_on(async {
        if let Some(connection_future) = &mut context.connection_future {
            if poll!(connection_future).is_ready() {
                context.connection_future = None;
            }
        }
    })
}

#[no_mangle]
extern "system" fn Join(context: &mut Context) {
    let context_ref = unsafe { &mut *(context as *mut Context) };

    context.connection_future = Some(Box::pin(connection(context_ref)));
}

type ConnectionFuture = dyn Future<Output = Result<(), Box<dyn Error>>>;

struct Context {
    mania_planet: &'static mut ManiaPlanet,
    connection_future: Option<Pin<Box<ConnectionFuture>>>,
    framed_tcp_stream: Option<FramedTcpStream>,
}

impl Context {
    fn new(mania_planet: &'static mut ManiaPlanet) -> Self {
        Self {
            mania_planet,
            connection_future: None,
            framed_tcp_stream: None,
        }
    }
}

async fn connection(context: &mut Context) -> Result<(), Box<dyn Error>> {
    let tcp_stream = TcpStream::connect("127.0.0.1:8369").compat().await?;

    let mut framed_tcp_stream = framed_tcp_stream(tcp_stream);

    let frame = framed_tcp_stream.try_next().await?.unwrap();
    let map_params_desc: MapParamsDesc = deserialize(&frame)?;

    open_map_editor(context, map_params_desc).await?;

    let frame = framed_tcp_stream.try_next().await?.unwrap();
    let map_desc: MapDesc = deserialize(&frame)?;

    while framed_tcp_stream.try_next().await?.is_some() {}

    Ok(())
}

async fn open_map_editor(
    context: &mut Context,
    params: MapParamsDesc,
) -> Result<(), Box<dyn Error>> {
    let process = Process::open_current()?;
    let main_module_memory = process.main_module_memory()?;

    let back_to_main_menu_fn = BackToMainMenuFn::find(&main_module_memory).unwrap();
    let edit_new_map_2_fn = EditNewMap2Fn::find(&main_module_memory).unwrap();

    let future = poll_fn(|_| {
        let module_stack = &context.mania_planet.switcher.module_stack;

        let editor_open = module_stack
            .iter()
            .any(|module| module.is_instance_of::<EditorCommon>());

        if editor_open {
            return Poll::Ready(());
        }

        if let Some(current_module) = module_stack.last() {
            if current_module.is_instance_of::<Menus>() {
                let mood_name = match params.mood {
                    Mood::Day => "Day",
                    Mood::Sunset => "Sunset",
                    Mood::Night => "Night",
                    Mood::Sunrise => "Sunrise",
                };

                let decoration_id = format!("48x48Screen155{mood_name}");

                unsafe {
                    edit_new_map_2_fn.call(
                        &mut context.mania_planet.mania_title_control_script_api,
                        &decoration_id,
                        "CarSport",
                    );
                };
            } else {
                unsafe {
                    back_to_main_menu_fn.call(context.mania_planet);
                }
            }
        }

        Poll::Pending
    });

    future.await;

    Ok(())
}
