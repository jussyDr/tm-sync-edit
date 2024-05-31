mod game;
mod process;

use std::panic;

use game::{edit_new_map, EditorCommon, ManiaPlanet};

#[no_mangle]
extern "system" fn Init(mania_planet: &'static mut ManiaPlanet) -> *mut Context {
    panic::set_hook(Box::new(|panic_info| {
        let _ = native_dialog::MessageDialog::new()
            .set_title("Error")
            .set_text(&panic_info.to_string())
            .show_alert();
    }));

    // let _ = native_dialog::MessageDialog::new()
    //     .set_title("Error")
    //     .set_text("editing...")
    //     .show_alert();

    unsafe { edit_new_map(&mania_planet.mania_title_control_script_api) };

    let context = Context::new(mania_planet);

    Box::into_raw(Box::new(context))
}

#[no_mangle]
unsafe extern "system" fn Destroy(context: *mut Context) {
    drop(Box::from_raw(context));
}

#[no_mangle]
extern "system" fn Update(context: &mut Context, editor: Option<&mut EditorCommon>) {}

struct Context {
    mania_planet: &'static mut ManiaPlanet,
}

impl Context {
    fn new(mania_planet: &'static mut ManiaPlanet) -> Self {
        Self { mania_planet }
    }
}
