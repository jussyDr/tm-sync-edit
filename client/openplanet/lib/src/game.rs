use std::{
    error::Error,
    ffi::c_void,
    io,
    mem::{size_of, transmute, MaybeUninit},
    ptr::{null, null_mut},
    slice,
};

use memchr::memmem;
use native_dialog::{MessageDialog, MessageType};
use windows_sys::Win32::{
    Foundation::{CloseHandle, FALSE},
    System::{
        Diagnostics::Debug::WriteProcessMemory,
        LibraryLoader::GetModuleHandleW,
        Memory::{
            VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
        },
        ProcessStatus::{GetModuleInformation, MODULEINFO},
        Threading::{
            GetCurrentProcess, GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION,
            PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
        },
    },
};

pub struct GameFns {
    place_block_fn: PlaceBlockFn,
    remove_block_fn: RemoveBlockFn,
    place_item_fn: PlaceItemFn,
    remove_item_fn: RemoveItemFn,
}

impl GameFns {
    pub fn find() -> Result<Self, Box<dyn Error>> {
        let process = unsafe { GetCurrentProcess() };
        let module = unsafe { GetModuleHandleW(null()) };

        let mut module_info = MaybeUninit::uninit();

        let success = unsafe {
            GetModuleInformation(
                process,
                module,
                module_info.as_mut_ptr(),
                size_of::<MODULEINFO>() as u32,
            )
        };

        if success == 0 {
            return Err(io::Error::last_os_error().into());
        }

        let module_info = unsafe { module_info.assume_init() };

        let module_memory = unsafe {
            slice::from_raw_parts(
                module_info.lpBaseOfDll as *const u8,
                module_info.SizeOfImage as usize,
            )
        };

        let place_block_fn_offset = memmem::find(
            module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x20, 0x4c, 0x89, 0x44, 0x24,
                0x18, 0x55,
            ],
        )
        .ok_or("failed to find place block function")?;

        let remove_block_fn_offset = memmem::find(
            module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x6c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24,
                0x18, 0x57, 0x48, 0x83, 0xec, 0x40, 0x83, 0x7c, 0x24, 0x70, 0x00,
            ],
        )
        .ok_or("failed to find remove block function")?;

        let place_item_fn_offset = memmem::find(
            module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x6c, 0x24, 0x18, 0x48, 0x89, 0x74, 0x24,
                0x20, 0x57, 0x48, 0x83, 0xec, 0x40, 0x49, 0x8b, 0xf9,
            ],
        )
        .ok_or("failed to find place item function")?;

        let remove_item_fn_offset = memmem::find(
            module_memory,
            &[
                0x48, 0x89, 0x5c, 0x24, 0x08, 0x57, 0x48, 0x83, 0xec, 0x30, 0x48, 0x8b, 0xfa, 0x48,
                0x8b, 0xd9, 0x48, 0x85, 0xd2, 0x0f, 0x84, 0xe6, 0x00, 0x00, 0x00,
            ],
        )
        .ok_or("failed to find remove item function")?;

        let place_block_fn =
            unsafe { transmute(module_memory.as_ptr().add(place_block_fn_offset)) };

        let remove_block_fn =
            unsafe { transmute(module_memory.as_ptr().add(remove_block_fn_offset)) };

        let place_item_fn = unsafe { transmute(module_memory.as_ptr().add(place_item_fn_offset)) };

        let remove_item_fn =
            unsafe { transmute(module_memory.as_ptr().add(remove_item_fn_offset)) };

        Ok(Self {
            place_block_fn,
            remove_block_fn,
            place_item_fn,
            remove_item_fn,
        })
    }

    fn place_block(&self) {
        unsafe { (self.place_block_fn)() }
    }

    fn place_free_block(&self) {
        unsafe { (self.place_block_fn)() }
    }

    fn remove_block(&self) {
        unsafe { (self.remove_block_fn)() }
    }

    fn place_item(&self) {
        unsafe { (self.place_item_fn)() }
    }

    fn remove_item(&self) {
        unsafe { (self.remove_item_fn)() }
    }
}

pub fn hook_place_block() -> Result<PlaceBlockHook, Box<dyn Error>> {
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

    let mut trampoline_code = [
        0x49, 0x8b, 0xe3, // mov rsp, r11
        0x41, 0x5f, // pop r15
        0x41, 0x5e, // pop r14
        0x41, 0x5d, // pop r13
        0x5f, // pop rdi
        0x5d, // pop rbp
        0x50, // push rax
        0x48, 0x89, 0xc1, // mov rcx, rax
        0x48, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, // mov rax, ????????
        0xff, 0xd0, // call rax
        0x58, // pop rax
        0xc3, // ret
    ];

    trampoline_code[17..17 + 8].copy_from_slice(&(place_block_callback as usize).to_le_bytes());

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

    let mut hook_code = [
        0x48, 0xb9, 0, 0, 0, 0, 0, 0, 0, 0, // mov rcx, ????????
        0xff, 0xe1, // jmp rcx
    ];

    hook_code[2..2 + 8].copy_from_slice(&(trampoline_ptr as usize).to_le_bytes());

    let mut n_written = MaybeUninit::uninit();

    let result = unsafe {
        WriteProcessMemory(
            current_process,
            exe_module_memory
                .as_ptr()
                .add(place_block_fn_end_offset + 16) as *const c_void,
            hook_code.as_ptr() as *const c_void,
            12,
            n_written.as_mut_ptr(),
        )
    };

    if result == 0 {
        return Err(io::Error::last_os_error().into());
    }

    let hook_ptr = unsafe {
        exe_module_memory
            .as_ptr()
            .add(place_block_fn_end_offset + 16) as *const c_void
    };

    unsafe {
        write_process_memory(
            current_process,
            hook_ptr,
            hook_code.as_ptr() as *const c_void,
            12,
        )?
    };

    unsafe { VirtualFree(trampoline_ptr, trampoline_code.len(), MEM_RELEASE) };

    unsafe { CloseHandle(current_process) };

    MessageDialog::new()
        .set_type(MessageType::Info)
        .set_title("SyncEdit.dll")
        .set_text("placed block!")
        .show_confirm()
        .unwrap();

    Ok(PlaceBlockHook { hook_ptr })
}

pub struct PlaceBlockHook {
    hook_ptr: *const c_void,
}

impl Drop for PlaceBlockHook {
    fn drop(&mut self) {
        let current_process = unsafe { open_current_process().unwrap() };

        let original_code: [u8; 12] = [
            0x49, 0x8b, 0xe3, 0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x5f, 0x5d, 0xc3,
        ];

        unsafe {
            write_process_memory(
                current_process,
                self.hook_ptr,
                original_code.as_ptr() as *const c_void,
                original_code.len(),
            )
            .unwrap()
        };
    }
}

unsafe extern "system" fn place_block_callback(block: *mut u8) {
    MessageDialog::new()
        .set_type(MessageType::Info)
        .set_title("SyncEdit.dll")
        .set_text("placed block!")
        .show_confirm()
        .unwrap();
}

pub fn hook_remove_block() -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn hook_place_item() -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn hook_remove_item() -> Result<(), Box<dyn Error>> {
    Ok(())
}

type PlaceBlockFn = unsafe extern "system" fn();

type RemoveBlockFn = unsafe extern "system" fn();

type PlaceItemFn = unsafe extern "system" fn();

type RemoveItemFn = unsafe extern "system" fn();

struct GameBlock {
    ptr: *mut u8,
}

impl GameBlock {}

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
    buffer: *const c_void,
    size: usize,
) -> io::Result<()> {
    let result = unsafe { WriteProcessMemory(process, base_addr, buffer, size, null_mut()) };

    if result == 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}
