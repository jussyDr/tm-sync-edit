mod os;

use std::{
    ffi::{c_char, c_void},
    iter,
    panic::{self, PanicInfo},
};

use os::{message_box, ModuleHandle, ProcessHandle};
use windows_sys::Win32::{
    Foundation::{BOOL, TRUE},
    System::SystemServices::{
        DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
    },
    UI::WindowsAndMessaging::{MB_ICONERROR, MB_ICONINFORMATION},
};

const PLACE_BLOCK_END_PATTERN: [u8; 28] = [
    0x4c, 0x8d, 0x9c, 0x24, 0xc0, 0x00, 0x00, 0x00, 0x49, 0x8b, 0x5b, 0x38, 0x49, 0x8b, 0x73, 0x48,
    0x49, 0x8b, 0xe3, 0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x5f, 0x5d, 0xc3,
];

#[no_mangle]
extern "system" fn DllMain(
    sync_edit_module: ModuleHandle,
    call_reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            panic::set_hook(Box::new(move |panic_info| {
                panic_hook(sync_edit_module, panic_info)
            }));
        }
        DLL_PROCESS_DETACH => {
            let _ = panic::take_hook();
        }
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {}
        _ => unreachable!(),
    }

    TRUE
}

#[no_mangle]
extern "system" fn Join(_host: *const c_char, _port: u16) {
    let current_process = ProcessHandle::open_current().unwrap();
    let trackmania_module = ModuleHandle::to_exe_file().unwrap();
    let trackmania_module_memory = current_process.module_memory(trackmania_module).unwrap();

    let place_block_end =
        find_unique_pattern(trackmania_module_memory, &PLACE_BLOCK_END_PATTERN).unwrap();

    message_box("wat de fak", "huh!!!", MB_ICONINFORMATION).unwrap();
}

#[no_mangle]
extern "system" fn Update() {}

fn panic_hook(sync_edit_module: ModuleHandle, panic_info: &PanicInfo) {
    let dll_file_name = sync_edit_module.get_file_name().unwrap();

    let text = panic_info.to_string();
    let caption = dll_file_name.file_name().unwrap().to_string_lossy();

    message_box(text, caption, MB_ICONERROR).unwrap();
}

fn encode_utf16_null_terminated(s: impl AsRef<str>) -> Vec<u16> {
    s.as_ref().encode_utf16().chain(iter::once(0)).collect()
}

fn find_unique_pattern(memory: &[u8], pattern: &[u8]) -> Option<usize> {
    let mut pattern_offset = None;

    for offset in 0..memory.len() - pattern.len() {
        let mut matches = true;

        for i in 0..pattern.len() {
            if memory[offset + i] != pattern[i] {
                matches = false;
                break;
            }
        }

        if matches {
            if pattern_offset.is_some() {
                return None;
            }

            pattern_offset = Some(offset);
        }
    }

    pattern_offset
}
