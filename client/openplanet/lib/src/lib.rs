#[cfg(not(all(target_arch = "x86_64", target_os = "windows")))]
compile_error!();

mod executable_buf;
mod hook;
mod process;

use std::{ffi::c_void, iter, panic};

use hook::RetHook;
use process::Process;
use windows_sys::Win32::{
    Foundation::{BOOL, HINSTANCE, TRUE},
    System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MESSAGEBOX_STYLE},
};

const PLACE_BLOCK_PATTERN: &[u8] = &[
    0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x20, 0x4c, 0x89, 0x44, 0x24, 0x18, 0x55,
];

#[no_mangle]
extern "system" fn DllMain(_instance: HINSTANCE, call_reason: u32, _reserved: *mut c_void) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            panic::set_hook(Box::new(|info| {
                message_box(&format!("{info}"), "Panic!", MB_ICONERROR).unwrap();
            }));
        }
        DLL_PROCESS_DETACH => {
            let _ = panic::take_hook();
        }
        _ => {}
    }

    TRUE
}

extern "win64" fn place_block_callback(_rax: u64) {}

#[no_mangle]
extern "C" fn Run() {
    unsafe {
        let process = Process::open_current().unwrap();

        let process_memory = process.memory();

        let place_block_offset =
            find_pattern(process_memory.as_slice(), PLACE_BLOCK_PATTERN).unwrap();

        let _place_block_hook = RetHook::hook(
            process_memory.slice(place_block_offset..),
            place_block_callback,
        )
        .unwrap();

        message_box("We have run successfully!", "Run", MB_ICONINFORMATION).unwrap();
    }
}

fn message_box(text: &str, caption: &str, ty: MESSAGEBOX_STYLE) -> Result<(), ()> {
    unsafe {
        let text = encode_nt_utf16(text);
        let caption = encode_nt_utf16(caption);

        let result = MessageBoxW(0, text.as_ptr(), caption.as_ptr(), ty);

        if result == 0 {
            return Err(());
        }

        Ok(())
    }
}

fn encode_nt_utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(iter::once(0)).collect()
}

fn find_pattern(slice: &[u8], pattern: &[u8]) -> Option<usize> {
    for i in 0..slice.len() - pattern.len() {
        let mut matches = true;

        for j in 0..pattern.len() {
            if slice[i + j] != pattern[j] {
                matches = false;
                break;
            }
        }

        if matches {
            return Some(i);
        }
    }

    None
}
