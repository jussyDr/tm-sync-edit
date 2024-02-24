//! Functionality for hooking functions.

use std::{
    io,
    sync::{Arc, Mutex},
};

use memchr::memmem;

use crate::os::{executable_page::ExecutablePage, process_memory::ProcessMemorySlice};

/// Error that occured when trying to hook a function.
#[derive(Debug)]
pub enum HookError {
    /// The target function could not be found.
    NotFound,
    /// An I/O error.
    Io(io::Error),
}

impl From<io::Error> for HookError {
    fn from(io_error: io::Error) -> Self {
        Self::Io(io_error)
    }
}

/// Hook the start of a function.
pub unsafe fn hook_start(
    exe_module_memory: &ProcessMemorySlice,
    executable_page: Arc<Mutex<ExecutablePage>>,
    code_pattern: &'static [u8],
    offset_in_pattern: usize,
    callback: usize,
    num_args: usize,
) -> Result<Hook, HookError> {
    let pattern_offset =
        memmem::find(exe_module_memory.as_slice(), code_pattern).ok_or(HookError::NotFound)?;

    let hooked_slice =
        exe_module_memory.slice(pattern_offset..pattern_offset + offset_in_pattern)?;

    let original_code = &code_pattern[..offset_in_pattern];

    let mut trampoline_code_prologue = vec![];

    trampoline_code_prologue.extend([
        0x51, // push rcx
        0x52, // push rdx
        0x41, 0x50, // push r8
        0x41, 0x51, // push r9
    ]);

    let num_stack_args = if num_args >= 4 { num_args - 4 } else { 0 };

    for _ in 0..num_stack_args {
        let dist = 32 + 8 + 32 + num_stack_args * 8;

        trampoline_code_prologue.extend([
            0xff, 0xb4, 0x24, 0, 0, 0, 0, // push [rsp + `dist`]
        ]);

        let vvv = trampoline_code_prologue.len();

        trampoline_code_prologue[vvv - 4..vvv].copy_from_slice(&(dist as u32).to_le_bytes());
    }

    let val = 32 + num_stack_args * 8;

    trampoline_code_prologue.extend([
        0x48, 0x83, 0xec, 0x20, // sub rsp, 32
        0x48, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, // mov rax, `callback`
        0xff, 0xd0, // call rax
        0x48, 0x83, 0xc4, 0, // add rsp, `val`
        0x41, 0x59, // pop r9
        0x41, 0x58, // pop r8
        0x5a, // pop rdx
        0x59, // pop rcx
    ]);

    let vvv = trampoline_code_prologue.len();
    trampoline_code_prologue[vvv - 20..vvv - 12].copy_from_slice(&(callback as u64).to_le_bytes());
    trampoline_code_prologue[vvv - 7..vvv - 6].copy_from_slice(&(val as u8).to_le_bytes());

    let mut trampoline_code_epilogue = [
        0x48, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, // mov rax, `end of hook_code`
        0xff, 0xe0, // jmp rax
    ];

    trampoline_code_epilogue[2..10].copy_from_slice(
        &(hooked_slice.as_slice().as_ptr().add(offset_in_pattern) as u64).to_le_bytes(),
    );

    let mut hook_code = [
        0x48, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, // mov rax, `trampoline`
        0xff, 0xe0, // jmp rax
        0x90, // nop
        0x90, // nop
        0x90, // nop
    ];

    {
        let mut ex_pa = executable_page.lock().unwrap();

        let trampoline = ex_pa.alloc(
            trampoline_code_prologue.len() + original_code.len() + trampoline_code_epilogue.len(),
        )?;

        trampoline[..trampoline_code_prologue.len()].copy_from_slice(&trampoline_code_prologue);

        trampoline
            [trampoline_code_prologue.len()..trampoline_code_prologue.len() + original_code.len()]
            .copy_from_slice(original_code);

        trampoline[trampoline_code_prologue.len() + original_code.len()
            ..trampoline_code_prologue.len()
                + original_code.len()
                + trampoline_code_epilogue.len()]
            .copy_from_slice(&trampoline_code_epilogue);

        hook_code[2..10].copy_from_slice(&(trampoline.as_ptr() as usize).to_le_bytes());

        unsafe { hooked_slice.write(&hook_code)? };
    }

    Ok(Hook {
        hooked_slice,
        original_code,
        _executable_page: executable_page,
    })
}

/// Hook the end of a function.
pub unsafe fn hook_end(
    exe_module_memory: &ProcessMemorySlice,
    executable_page: Arc<Mutex<ExecutablePage>>,
    code_pattern: &'static [u8],
    offset_in_pattern: usize,
    callback: usize,
) -> Result<Hook, HookError> {
    let original_code = &code_pattern[offset_in_pattern..];

    let pattern_offset =
        memmem::find(exe_module_memory.as_slice(), code_pattern).ok_or(HookError::NotFound)?;

    let offset = pattern_offset + offset_in_pattern;

    let hooked_slice = exe_module_memory
        .slice(offset..offset + original_code.len())
        .unwrap();

    let mut trampoline_code = [
        0x50, // push rax
        0x48, 0x89, 0xc1, // mov rcx, rax
        0x48, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, // mov rax, `callback`
        0xff, 0xd0, // call rax
        0x58, // pop rax
        0xc3, // ret
    ];

    trampoline_code[6..14].copy_from_slice(&(callback as u64).to_le_bytes());

    {
        let mut ex_pa = executable_page.lock().unwrap();

        let trampoline = ex_pa.alloc(original_code.len() + trampoline_code.len())?;

        trampoline[..original_code.len() - 1]
            .copy_from_slice(&original_code[..original_code.len() - 1]);

        trampoline[original_code.len() - 1..original_code.len() - 1 + trampoline_code.len()]
            .copy_from_slice(&trampoline_code);

        let mut hook_code = [
            0x48, 0xb9, 0, 0, 0, 0, 0, 0, 0, 0, // mov rcx, `trampoline`
            0xff, 0xe1, // jmp rcx
        ];

        hook_code[2..10].copy_from_slice(&(trampoline.as_ptr() as usize).to_le_bytes());

        unsafe { hooked_slice.write(&hook_code)? };
    }

    Ok(Hook {
        hooked_slice,
        original_code,
        _executable_page: executable_page,
    })
}

/// Represents a hooked function.
///
/// The function will automatically be unhooked when dropped.
pub struct Hook {
    /// Slice of memory that has been overwritten by the hook code.
    hooked_slice: ProcessMemorySlice,
    /// Original code that has been overwritten by the hook code.
    original_code: &'static [u8],
    /// Reference to the executable page which contains this hook's trampoline code.
    _executable_page: Arc<Mutex<ExecutablePage>>,
}

impl Drop for Hook {
    fn drop(&mut self) {
        unsafe { self.hooked_slice.write(self.original_code).unwrap() };
    }
}
