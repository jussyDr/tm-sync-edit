//! Functionality for hooking into the game.

use std::{
    error::Error,
    ffi::c_void,
    io,
    mem::{size_of, MaybeUninit},
    ptr::{null, null_mut},
    slice,
};

use memchr::memmem;
use windows_sys::Win32::{
    Foundation::{CloseHandle, FALSE},
    System::{
        Diagnostics::Debug::WriteProcessMemory,
        LibraryLoader::GetModuleHandleW,
        Memory::{VirtualAlloc, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE},
        ProcessStatus::{GetModuleInformation, MODULEINFO},
        Threading::{
            GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION,
            PROCESS_VM_READ, PROCESS_VM_WRITE,
        },
    },
};

use super::{Block, Item, ItemParams};

pub type PlaceBlockCallbackFn = unsafe extern "system" fn(*mut u8, *mut Block);

pub type RemoveBlockCallbackFn = unsafe extern "system" fn(*mut u8, *mut Block);

pub type PlaceItemCallbackFn = unsafe extern "system" fn(*mut u8, *mut ItemParams);

pub type RemoveItemCallbackFn = unsafe extern "system" fn(*mut u8, *mut Item);

pub struct Hook {
    ptr: *const u8,
    original_code: &'static [u8],
}

impl Drop for Hook {
    fn drop(&mut self) {
        let current_process = unsafe { open_current_process().unwrap() };

        unsafe {
            write_process_memory(
                current_process,
                self.ptr as *const c_void,
                self.original_code,
            )
            .unwrap()
        };
    }
}

fn hook(
    code_pattern: &[u8],
    code_pattern_offset: usize,
    original_code: &'static [u8],
    trampoline_code_fn: impl Fn(*const u8) -> Vec<u8>,
    hook_code_fn: impl Fn(*const u8) -> Vec<u8>,
) -> Result<Hook, Box<dyn Error>> {
    let current_process = unsafe { open_current_process()? };

    let exe_module = unsafe { GetModuleHandleW(null()) };

    let mut exe_module_info = MaybeUninit::uninit();

    let success = unsafe {
        GetModuleInformation(
            current_process,
            exe_module,
            exe_module_info.as_mut_ptr(),
            size_of::<MODULEINFO>() as u32,
        )
    };

    if success == 0 {
        return Err(io::Error::last_os_error().into());
    }

    let exe_module_info = unsafe { exe_module_info.assume_init() };

    let exe_module_memory = unsafe {
        slice::from_raw_parts(
            exe_module_info.lpBaseOfDll as *const u8,
            exe_module_info.SizeOfImage as usize,
        )
    };

    let hook_offset = memmem::find(exe_module_memory, code_pattern)
        .ok_or("failed to find code pattern")?
        + code_pattern_offset;

    let hook_ptr = unsafe { exe_module_memory.as_ptr().add(hook_offset) };
    let hook_end_ptr = unsafe { hook_ptr.add(original_code.len()) };

    let trampoline_code = trampoline_code_fn(hook_end_ptr);

    let trampoline_ptr = unsafe {
        VirtualAlloc(
            null(),
            trampoline_code.len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        )
    };

    if trampoline_ptr.is_null() {
        return Err(io::Error::last_os_error().into());
    }

    let trampoline =
        unsafe { slice::from_raw_parts_mut(trampoline_ptr as *mut u8, trampoline_code.len()) };

    trampoline.copy_from_slice(&trampoline_code);

    let hook_code = hook_code_fn(trampoline_ptr as *const u8);

    unsafe { write_process_memory(current_process, hook_ptr as *const c_void, &hook_code)? };

    unsafe { CloseHandle(current_process) };

    Ok(Hook {
        ptr: hook_ptr,
        original_code,
    })
}

pub fn hook_place_block(
    user_data: *mut u8,
    callback: PlaceBlockCallbackFn,
) -> Result<Hook, Box<dyn Error>> {
    let code_pattern = &[
        0x4c, 0x8d, 0x9c, 0x24, 0xc0, 0x00, 0x00, 0x00, 0x49, 0x8b, 0x5b, 0x38, 0x49, 0x8b, 0x73,
        0x48, 0x49, 0x8b, 0xe3, 0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x5f, 0x5d, 0xc3,
    ];

    let original_code = &code_pattern[16..];

    let trampoline_code_fn = |_hook_end_ptr| {
        let mut trampoline_code = vec![];

        trampoline_code.extend_from_slice(&original_code[..11]);

        trampoline_code.extend_from_slice(&[
            0x50, // push rax
            0x48, 0xb9, // mov rcx, ????????
        ]);

        trampoline_code.extend_from_slice(&(user_data as usize).to_le_bytes());

        trampoline_code.extend_from_slice(&[
            0x48, 0x89, 0xc2, // mov rdx, rax
            0x48, 0xb8, // mov rax, ????????
        ]);

        trampoline_code.extend_from_slice(&(callback as usize).to_le_bytes());

        trampoline_code.extend_from_slice(&[
            0xff, 0xd0, // call rax
            0x58, // pop rax
            0xc3, // ret
        ]);

        trampoline_code
    };

    let hook_code_fn = |trampoline_ptr| {
        let mut hook_code = vec![];

        hook_code.extend_from_slice(&[
            0x48, 0xb9, // mov rcx, ????????
        ]);

        hook_code.extend_from_slice(&(trampoline_ptr as usize).to_le_bytes());

        hook_code.extend_from_slice(&[
            0xff, 0xe1, // jmp rcx
        ]);

        hook_code
    };

    hook(
        code_pattern,
        16,
        original_code,
        trampoline_code_fn,
        hook_code_fn,
    )
}

pub fn hook_remove_block(
    user_data: *mut u8,
    callback: RemoveBlockCallbackFn,
) -> Result<Hook, Box<dyn Error>> {
    let code_pattern = &[
        0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x6c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x18,
        0x57, 0x48, 0x83, 0xec, 0x40, 0x83, 0x7c, 0x24, 0x70, 0x00,
    ];

    let original_code = &code_pattern[..15];

    let trampoline_code_fn = |hook_end_ptr| {
        let mut trampoline_code = vec![];

        trampoline_code.extend_from_slice(original_code);

        trampoline_code.extend_from_slice(&[
            0x51, // push rcx
            0x52, // push rdx
            0x41, 0x50, // push r8
            0x41, 0x51, // push r9
            0x48, 0x83, 0xec, 0x28, // sub rsp, 40
            0x48, 0xb9, // mov rcx, ????????
        ]);

        trampoline_code.extend_from_slice(&(user_data as usize).to_le_bytes());

        trampoline_code.extend_from_slice(&[
            0x48, 0xb8, // mov rax, ????????
        ]);

        trampoline_code.extend_from_slice(&(callback as usize).to_le_bytes());

        trampoline_code.extend_from_slice(&[
            0xff, 0xd0, // call rax
            0x48, 0x83, 0xc4, 0x28, // add rsp, 40
            0x41, 0x59, // pop r9
            0x41, 0x58, // pop r8
            0x5a, // pop rdx
            0x59, // pop rcx
            0x48, 0xb8, // mov rax, ????????
        ]);

        trampoline_code.extend_from_slice(&(hook_end_ptr as usize).to_le_bytes());

        trampoline_code.extend_from_slice(&[
            0xff, 0xe0, // jmp rax
        ]);

        trampoline_code
    };

    let hook_code_fn = |trampoline_ptr| {
        let mut hook_code = vec![];

        hook_code.extend_from_slice(&[
            0x48, 0xb8, // mov rax, ????????
        ]);

        hook_code.extend_from_slice(&(trampoline_ptr as usize).to_le_bytes());

        hook_code.extend_from_slice(&[
            0xff, 0xe0, // jmp rax
        ]);

        hook_code
    };

    hook(
        code_pattern,
        0,
        original_code,
        trampoline_code_fn,
        hook_code_fn,
    )
}

pub fn hook_place_item(
    user_data: *mut u8,
    callback: PlaceItemCallbackFn,
) -> Result<Hook, Box<dyn Error>> {
    let code_pattern = &[
        0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x6c, 0x24, 0x18, 0x48, 0x89, 0x74, 0x24, 0x20,
        0x57, 0x48, 0x83, 0xec, 0x40, 0x49, 0x8b, 0xf9,
    ];

    let original_code = &code_pattern[..15];

    let trampoline_code_fn = |hook_end_ptr| {
        let mut trampoline_code = vec![];

        trampoline_code.extend_from_slice(original_code);

        trampoline_code.extend_from_slice(&[
            0x51, // push rcx
            0x52, // push rdx
            0x41, 0x50, // push r8
            0x41, 0x51, // push r9
            0x48, 0x83, 0xec, 0x28, // sub rsp, 40
            0x48, 0xb9, // mov rcx, ????????
        ]);

        trampoline_code.extend_from_slice(&(user_data as usize).to_le_bytes());

        trampoline_code.extend_from_slice(&[
            0x48, 0xb8, // mov rax, ????????
        ]);

        trampoline_code.extend_from_slice(&(callback as usize).to_le_bytes());

        trampoline_code.extend_from_slice(&[
            0xff, 0xd0, // call rax
            0x48, 0x83, 0xc4, 0x28, // add rsp, 40
            0x41, 0x59, // pop r9
            0x41, 0x58, // pop r8
            0x5a, // pop rdx
            0x59, // pop rcx
            0x48, 0xb8, // mov rax, ????????
        ]);

        trampoline_code.extend_from_slice(&(hook_end_ptr as usize).to_le_bytes());

        trampoline_code.extend_from_slice(&[
            0xff, 0xe0, // jmp rax
        ]);

        trampoline_code
    };

    let hook_code_fn = |trampoline_ptr| {
        let mut hook_code = vec![];

        hook_code.extend_from_slice(&[
            0x48, 0xb8, // mov rax, ????????
        ]);

        hook_code.extend_from_slice(&(trampoline_ptr as usize).to_le_bytes());

        hook_code.extend_from_slice(&[
            0xff, 0xe0, // jmp rax
        ]);

        hook_code
    };

    hook(
        code_pattern,
        0,
        original_code,
        trampoline_code_fn,
        hook_code_fn,
    )
}

pub fn hook_remove_item(
    user_data: *mut u8,
    callback: RemoveItemCallbackFn,
) -> Result<Hook, Box<dyn Error>> {
    let code_pattern = &[
        0x48, 0x89, 0x5c, 0x24, 0x08, 0x57, 0x48, 0x83, 0xec, 0x30, 0x48, 0x8b, 0xfa, 0x48, 0x8b,
        0xd9, 0x48, 0x85, 0xd2, 0x0f, 0x84, 0xe6, 0x00, 0x00, 0x00,
    ];

    let original_code = &code_pattern[..13];

    let trampoline_code_fn = |hook_end_ptr| {
        let mut trampoline_code = vec![];

        trampoline_code.extend_from_slice(original_code);

        trampoline_code.extend_from_slice(&[
            0x51, // push rcx
            0x52, // push rdx
            0x41, 0x50, // push r8
            0x41, 0x51, // push r9
            0x48, 0x83, 0xec, 0x20, // sub rsp, 32
            0x48, 0xb9, // mov rcx, ????????
        ]);

        trampoline_code.extend_from_slice(&(user_data as usize).to_le_bytes());

        trampoline_code.extend_from_slice(&[
            0x48, 0xb8, // mov rax, ????????
        ]);

        trampoline_code.extend_from_slice(&(callback as usize).to_le_bytes());

        trampoline_code.extend_from_slice(&[
            0xff, 0xd0, // call rax
            0x48, 0x83, 0xc4, 0x20, // add rsp, 32
            0x41, 0x59, // pop r9
            0x41, 0x58, // pop r8
            0x5a, // pop rdx
            0x59, // pop rcx
            0x48, 0xb8, // mov rax, ????????
        ]);

        trampoline_code.extend_from_slice(&(hook_end_ptr as usize).to_le_bytes());

        trampoline_code.extend_from_slice(&[
            0xff, 0xe0, // jmp rax
        ]);

        trampoline_code
    };

    let hook_code_fn = |trampoline_ptr| {
        let mut hook_code = vec![];

        hook_code.extend_from_slice(&[
            0x48, 0xb8, // mov rax, ????????
        ]);

        hook_code.extend_from_slice(&(trampoline_ptr as usize).to_le_bytes());

        hook_code.extend_from_slice(&[
            0xff, 0xe0, // jmp rax
        ]);

        hook_code
    };

    hook(
        code_pattern,
        0,
        original_code,
        trampoline_code_fn,
        hook_code_fn,
    )
}

unsafe fn open_current_process() -> io::Result<isize> {
    let current_process_id = unsafe { GetCurrentProcessId() };

    let current_process = unsafe {
        OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE,
            FALSE,
            current_process_id,
        )
    };

    if current_process == 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(current_process)
}

unsafe fn write_process_memory(
    process: isize,
    base_addr: *const c_void,
    buf: &[u8],
) -> io::Result<()> {
    let result = unsafe {
        WriteProcessMemory(
            process,
            base_addr,
            buf.as_ptr() as *const c_void,
            buf.len(),
            null_mut(),
        )
    };

    if result == 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}
