use std::{panic, sync::Mutex, thread};

use native_dialog::{MessageDialog, MessageType};
use tokio::{runtime, select, sync::oneshot};
use windows_sys::Win32::{
    Foundation::{BOOL, TRUE},
    System::SystemServices::DLL_PROCESS_ATTACH,
};

static SHOULD_JOIN: Mutex<bool> = Mutex::new(false);

static JOIN_SUCCESS_SENDER: Mutex<Option<oneshot::Sender<bool>>> = Mutex::new(None);

#[no_mangle]
extern "system" fn DllMain(_dll_module: usize, call_reason: u32, _reserved: usize) -> BOOL {
    if call_reason == DLL_PROCESS_ATTACH {
        panic::set_hook(Box::new(|panic_info| {
            MessageDialog::new()
                .set_type(MessageType::Error)
                .set_title("SyncEdit.dll")
                .set_text(&panic_info.to_string())
                .show_alert()
                .unwrap();
        }));
    }

    TRUE
}

#[no_mangle]
extern "system" fn Join() {
    thread::spawn(|| {
        let runtime = runtime::Builder::new_current_thread().build().unwrap();

        runtime.block_on(async {
            let (join_success_sender, join_success_receiver) = oneshot::channel();

            *JOIN_SUCCESS_SENDER.lock().unwrap() = Some(join_success_sender);

            *SHOULD_JOIN.lock().unwrap() = true;

            let join_success = join_success_receiver.await.unwrap();

            MessageDialog::new()
                .set_type(MessageType::Info)
                .set_title("SyncEdit.dll")
                .set_text(&join_success.to_string())
                .show_alert()
                .unwrap();
        });
    });
}

#[no_mangle]
extern "system" fn ShouldJoin() -> bool {
    let mut should_join = SHOULD_JOIN.lock().unwrap();

    if *should_join {
        *should_join = false;

        true
    } else {
        false
    }
}

#[no_mangle]
extern "system" fn JoinSuccess(success: bool) {
    JOIN_SUCCESS_SENDER
        .lock()
        .unwrap()
        .take()
        .unwrap()
        .send(success)
        .unwrap();
}
