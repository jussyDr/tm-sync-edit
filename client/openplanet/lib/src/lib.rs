mod os;

use std::{
    ffi::c_void,
    mem::{size_of, MaybeUninit},
    panic,
    ptr::{self, null, NonNull},
    slice,
    sync::Mutex,
};

use os::{message_box, DllCallReason, MessageBoxType};
use windows_sys::Win32::{
    Foundation::{BOOL, FALSE, TRUE},
    System::{
        LibraryLoader::GetModuleHandleW,
        Memory::{
            VirtualAlloc, VirtualFree, VirtualProtectEx, MEM_COMMIT, MEM_DECOMMIT, MEM_RELEASE,
            MEM_RESERVE, PAGE_EXECUTE_READWRITE,
        },
        ProcessStatus::{GetModuleInformation, MODULEINFO},
        SystemInformation::GetSystemInfo,
        Threading::{
            GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION,
            PROCESS_VM_READ, PROCESS_VM_WRITE,
        },
    },
};

static STATE: Mutex<Option<State>> = Mutex::new(None);

struct State {
    current_process: isize,
    ptr: isize,
    executable_page: ExecutablePage,
}

#[no_mangle]
extern "system" fn DllMain(_module: isize, call_reason: DllCallReason, _reserved: isize) -> BOOL {
    match call_reason {
        DllCallReason::ProcessAttach => {
            panic::set_hook(Box::new(|panic_info| {
                message_box(
                    "SyncEdit.dll",
                    &panic_info.to_string(),
                    MessageBoxType::Error,
                )
                .unwrap();
            }));
        }
        DllCallReason::ProcessDetach => {
            let _ = panic::take_hook();
        }
        _ => {}
    }

    TRUE
}

#[no_mangle]
extern "system" fn Init() {
    let current_process_id = unsafe { GetCurrentProcessId() };

    let current_process = unsafe {
        OpenProcess(
            PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_QUERY_INFORMATION,
            FALSE,
            current_process_id,
        )
    };

    let exe_module = unsafe { GetModuleHandleW(null()) };

    let mut exe_module_info = MaybeUninit::uninit();

    unsafe {
        GetModuleInformation(
            current_process,
            exe_module,
            exe_module_info.as_mut_ptr(),
            size_of::<MODULEINFO>() as u32,
        )
    };

    let exe_module_info = unsafe { exe_module_info.assume_init() };

    let exe_module_memory = unsafe {
        slice::from_raw_parts(
            exe_module_info.lpBaseOfDll as *const u8,
            exe_module_info.SizeOfImage as usize,
        )
    };

    let offset = find_pattern(
        exe_module_memory,
        &[
            0x4c, 0x8d, 0x9c, 0x24, 0xc0, 0x00, 0x00, 0x00, 0x49, 0x8b, 0x5b, 0x38, 0x49, 0x8b,
            0x73, 0x48, 0x49, 0x8b, 0xe3, 0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x5f, 0x5d, 0xc3,
        ],
    )
    .unwrap();

    let mut system_info = MaybeUninit::uninit();

    unsafe { GetSystemInfo(system_info.as_mut_ptr()) };

    let system_info = unsafe { system_info.assume_init() };

    let executable_page = unsafe {
        VirtualAlloc(
            null(),
            system_info.dwPageSize as usize,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        )
    };

    let mut trampoline = [
        0x49, 0x8b, 0xe3, 0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x5f, 0x5d, 0x50, 0x48, 0x89, 0xc1,
        0x48, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xd0, 0x58, 0xc3,
    ];

    trampoline[17..25].copy_from_slice(&(callback as usize).to_le_bytes());

    unsafe {
        ptr::copy_nonoverlapping(
            trampoline.as_ptr(),
            executable_page as *mut u8,
            trampoline.len(),
        )
    };

    let ptr = unsafe { exe_module_memory.as_ptr().add(offset + 16) };

    let mut old_protect = MaybeUninit::uninit();

    unsafe {
        VirtualProtectEx(
            current_process,
            ptr as *const c_void,
            12,
            PAGE_EXECUTE_READWRITE,
            old_protect.as_mut_ptr(),
        )
    };

    let mut hook = [0x48, 0xb9, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xe1];

    hook[2..10].copy_from_slice(&(executable_page as usize).to_le_bytes());

    unsafe {
        ptr::copy_nonoverlapping(hook.as_ptr(), ptr as *mut u8, hook.len());
    }

    let executable_page = ExecutablePage {
        ptr: NonNull::new(executable_page as *mut u8).unwrap(),
        size: system_info.dwPageSize as usize,
    };

    *STATE.lock().unwrap() = Some(State {
        current_process,
        ptr: ptr as isize,
        executable_page,
    });

    message_box("SyncEdit.dll", "initialized", MessageBoxType::Info).unwrap();
}

#[no_mangle]
extern "system" fn Destroy() {
    {
        let state = STATE.lock().unwrap();
        let state = state.as_ref().unwrap();

        let code: [u8; 12] = [
            0x49, 0x8b, 0xe3, 0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x5f, 0x5d, 0xc3,
        ];

        unsafe {
            ptr::copy_nonoverlapping(code.as_ptr(), state.ptr as *mut u8, code.len());
        }

        unsafe {
            VirtualFree(
                state.executable_page.ptr.as_ptr() as *mut c_void,
                state.executable_page.size,
                MEM_DECOMMIT | MEM_RELEASE,
            )
        };
    }

    // *STATE.lock().unwrap() = None;

    message_box("SyncEdit.dll", "destroyed", MessageBoxType::Info).unwrap();
}

extern "system" fn callback(_rax: u64) {
    message_box("SyncEdit.dll", "callback", MessageBoxType::Info).unwrap();
}

struct ExecutablePage {
    ptr: NonNull<u8>,
    size: usize,
}

unsafe impl Send for ExecutablePage {}

unsafe impl Sync for ExecutablePage {}

fn find_pattern(memory: &[u8], pattern: &[u8]) -> Option<usize> {
    for offset in 0..memory.len() - pattern.len() {
        let mut matches = true;

        for i in 0..pattern.len() {
            if memory[offset + i] != pattern[i] {
                matches = false;
                break;
            }
        }

        if matches {
            return Some(offset);
        }
    }

    None
}
