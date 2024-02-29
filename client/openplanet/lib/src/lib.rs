mod game;
mod hook;
mod windows;

use std::{ffi::c_void, io::Result, panic, sync::Mutex};

use game::Fns;
use windows::{message_box, DllCallReason, MessageBoxType};
use windows_sys::Win32::Foundation::{BOOL, HINSTANCE, TRUE};

static STATE: Mutex<State> = Mutex::new(State::new());

struct State {
    game_fns: Option<Fns>,
}

impl State {
    const fn new() -> Self {
        Self { game_fns: None }
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

fn init() -> Result<()> {
    let mut state = STATE.lock().unwrap();

    state.game_fns = unsafe { Some(Fns::find()?) };

    Ok(())
}

#[no_mangle]
extern "C" fn Destroy() {
    destroy().unwrap();
}

fn destroy() -> Result<()> {
    let mut state = STATE.lock().unwrap();

    state.game_fns = None;

    Ok(())
}
