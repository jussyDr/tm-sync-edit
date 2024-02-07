use std::{
    ffi::{c_void, OsString},
    io, iter,
    os::windows::ffi::OsStringExt,
    panic,
    path::PathBuf,
};

use windows_sys::Win32::{
    Foundation::{BOOL, HINSTANCE, HMODULE, TRUE},
    System::{
        LibraryLoader::GetModuleFileNameW,
        SystemServices::{
            DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
        },
    },
    UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MESSAGEBOX_STYLE},
};

#[no_mangle]
extern "system" fn DllMain(
    dll_handle: HINSTANCE,
    call_reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            panic::set_hook(Box::new(move |panic_info| {
                let dll_file_name = get_module_file_name(dll_handle).unwrap();

                let text = format!("{panic_info}");
                let caption = dll_file_name.file_name().unwrap().to_string_lossy();

                message_box(text, caption, MB_ICONERROR).unwrap();
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
extern "system" fn update() {}

fn encode_utf16_null_terminated(s: impl AsRef<str>) -> Vec<u16> {
    s.as_ref().encode_utf16().chain(iter::once(0)).collect()
}

fn message_box(
    text: impl AsRef<str>,
    caption: impl AsRef<str>,
    ty: MESSAGEBOX_STYLE,
) -> io::Result<()> {
    let text = encode_utf16_null_terminated(text);
    let caption = encode_utf16_null_terminated(caption);

    let result = unsafe { MessageBoxW(0, text.as_ptr(), caption.as_ptr(), ty) };

    if result == 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

fn get_module_file_name(module: HMODULE) -> io::Result<PathBuf> {
    let mut module_file_name = vec![0; 128];

    let result = unsafe {
        GetModuleFileNameW(
            module,
            module_file_name.as_mut_ptr(),
            module_file_name.len() as u32,
        )
    };

    if result == 0 {
        return Err(io::Error::last_os_error());
    }

    let module_file_name = OsString::from_wide(&module_file_name[..result as usize]);

    Ok(module_file_name.into())
}
