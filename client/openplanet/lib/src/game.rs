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

pub type PlaceBlockCallbackFn = unsafe extern "system" fn(*const u8);

pub type RemoveBlockCallbackFn = unsafe extern "system" fn(*const u8);

pub type PlaceItemCallbackFn = unsafe extern "system" fn(*const u8);

pub type RemoveItemCallbackFn = unsafe extern "system" fn(*const u8);

pub fn hook_place_block(callback: PlaceBlockCallbackFn) -> Result<Hook, Box<dyn Error>> {
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

    let place_block_fn_end_offset = memmem::find(
        exe_module_memory,
        &[
            0x4c, 0x8d, 0x9c, 0x24, 0xc0, 0x00, 0x00, 0x00, 0x49, 0x8b, 0x5b, 0x38, 0x49, 0x8b,
            0x73, 0x48, 0x49, 0x8b, 0xe3, 0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x5f, 0x5d, 0xc3,
        ],
    )
    .ok_or("failed to find place block function end")?;

    let trampoline_code = {
        let mut trampoline_code = vec![];

        trampoline_code.extend_from_slice(&[
            0x49, 0x8b, 0xe3, // mov rsp, r11
            0x41, 0x5f, // pop r15
            0x41, 0x5e, // pop r14
            0x41, 0x5d, // pop r13
            0x5f, // pop rdi
            0x5d, // pop rbp
            0x50, // push rax
            0x48, 0x89, 0xc1, // mov rcx, rax
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

    let hook_code = {
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

    let hook_ptr = unsafe {
        exe_module_memory
            .as_ptr()
            .add(place_block_fn_end_offset + 16) as *const c_void
    };

    unsafe { write_process_memory(current_process, hook_ptr, &hook_code)? };

    unsafe { CloseHandle(current_process) };

    Ok(Hook {
        ptr: hook_ptr as *const u8,
        original_code: &[
            0x49, 0x8b, 0xe3, 0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x5f, 0x5d, 0xc3,
        ],
    })
}

pub fn hook_remove_block(callback: RemoveBlockCallbackFn) -> Result<Hook, Box<dyn Error>> {
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

    let remove_block_fn_offset = memmem::find(
        exe_module_memory,
        &[
            0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x6c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24,
            0x18, 0x57, 0x48, 0x83, 0xec, 0x40, 0x83, 0x7c, 0x24, 0x70, 0x00,
        ],
    )
    .ok_or("failed to find remove block function")?;

    let hook_ptr = unsafe { exe_module_memory.as_ptr().add(remove_block_fn_offset) };
    let hook_end_ptr = unsafe { hook_ptr.add(15) };

    let trampoline_code = {
        let mut trampoline_code = vec![];

        trampoline_code.extend_from_slice(&[
            0x48, 0x89, 0x5c, 0x24, 0x08, // mov [rsp + 8], rbx
            0x48, 0x89, 0x6c, 0x24, 0x10, // mov [rsp + 16], rbp
            0x48, 0x89, 0x74, 0x24, 0x18, // mov [rsp + 24], rsi
            0x51, // push rcx
            0x52, // push rdx
            0x41, 0x50, // push r8
            0x41, 0x51, // push r9
            0x48, 0x83, 0xec, 0x20, // sub rsp, 32
            0x48, 0x89, 0xd1, // mov rcx, rdx
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

    let hook_code = {
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

    unsafe { write_process_memory(current_process, hook_ptr as *const c_void, &hook_code)? };

    unsafe { CloseHandle(current_process) };

    Ok(Hook {
        ptr: hook_ptr,
        original_code: &[
            0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x6c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24,
            0x18,
        ],
    })
}

pub fn hook_place_item(callback: PlaceItemCallbackFn) -> Result<Hook, Box<dyn Error>> {
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

    let place_item_fn_offset = memmem::find(
        exe_module_memory,
        &[
            0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x6c, 0x24, 0x18, 0x48, 0x89, 0x74, 0x24,
            0x20, 0x57, 0x48, 0x83, 0xec, 0x40, 0x49, 0x8b, 0xf9,
        ],
    )
    .ok_or("failed to find place item function")?;

    let hook_ptr = unsafe { exe_module_memory.as_ptr().add(place_item_fn_offset) };
    let hook_end_ptr = unsafe { hook_ptr.add(15) };

    let trampoline_code = {
        let mut trampoline_code = vec![];

        trampoline_code.extend_from_slice(&[
            0x48, 0x89, 0x5c, 0x24, 0x10, // mov [rsp + 16], rbx
            0x48, 0x89, 0x6c, 0x24, 0x18, // mov [rsp + 24], rbp
            0x48, 0x89, 0x74, 0x24, 0x20, // mov [rsp + 32], rsi
            0x51, // push rcx
            0x52, // push rdx
            0x41, 0x50, // push r8
            0x41, 0x51, // push r9
            0x48, 0x83, 0xec, 0x20, // sub rsp, 32
            0x48, 0x89, 0xd1, // mov rcx, rdx
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

    let hook_code = {
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

    unsafe { write_process_memory(current_process, hook_ptr as *const c_void, &hook_code)? };

    unsafe { CloseHandle(current_process) };

    Ok(Hook {
        ptr: hook_ptr,
        original_code: &[
            0x48, 0x89, 0x5c, 0x24, 0x10, // mov [rsp + 16], rbx
            0x48, 0x89, 0x6c, 0x24, 0x18, // mov [rsp + 24], rbp
            0x48, 0x89, 0x74, 0x24, 0x20, // mov [rsp + 32], rsi
        ],
    })
}

pub fn hook_remove_item(callback: RemoveItemCallbackFn) -> Result<Hook, Box<dyn Error>> {
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

    let remove_item_fn_offset = memmem::find(
        exe_module_memory,
        &[
            0x48, 0x89, 0x5c, 0x24, 0x08, 0x57, 0x48, 0x83, 0xec, 0x30, 0x48, 0x8b, 0xfa, 0x48,
            0x8b, 0xd9, 0x48, 0x85, 0xd2, 0x0f, 0x84, 0xe6, 0x00, 0x00, 0x00,
        ],
    )
    .ok_or("failed to find remove item function")?;

    let hook_ptr = unsafe { exe_module_memory.as_ptr().add(remove_item_fn_offset) };
    let hook_end_ptr = unsafe { hook_ptr.add(15) };

    let trampoline_code = {
        let mut trampoline_code = vec![];

        trampoline_code.extend_from_slice(&[
            0x48, 0x89, 0x5c, 0x24, 0x08, // mov [rsp + 8], rbx
            0x57, // push rdi
            0x48, 0x83, 0xec, 0x30, // sub rsp, 48
            0x48, 0x8b, 0xfa, // mov rdi, rdx
            0x51, // push rcx
            0x52, // push rdx
            0x41, 0x50, // push r8
            0x41, 0x51, // push r9
            0x48, 0x83, 0xec, 0x20, // sub rsp, 32
            0x48, 0x89, 0xd1, // mov rcx, rdx
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

    let hook_code = {
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

    unsafe { write_process_memory(current_process, hook_ptr as *const c_void, &hook_code)? };

    unsafe { CloseHandle(current_process) };

    Ok(Hook {
        ptr: hook_ptr,
        original_code: &[
            0x48, 0x89, 0x5c, 0x24, 0x08, // mov [rsp + 8], rbx
            0x57, // push rdi
            0x48, 0x83, 0xec, 0x30, // sub rsp, 48
            0x48, 0x8b, 0xfa, // mov rdi, rdx
        ],
    })
}

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
