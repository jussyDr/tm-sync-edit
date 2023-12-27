use iced_x86::{
    code_asm::{ptr, rax, rcx, rsp, CodeAssembler},
    BlockEncoder, BlockEncoderOptions, Decoder, DecoderOptions, IcedError, InstructionBlock,
};

use crate::{executable_buf::ExecutableBuf, process::ProcessMemory};

const BITNESS: u32 = 64;

/// Hook that is called when a function returns.
pub struct RetHook<'a> {
    memory: ProcessMemory<'a>,
    original_code: Vec<u8>,
    _hook: ExecutableBuf,
    _trampoline: ExecutableBuf,
}

impl<'a> RetHook<'a> {
    /// Hook the `callback` in function at the given `offset` in the given `process`.
    pub fn hook(
        mut memory: ProcessMemory<'a>,
        callback: extern "win64" fn(u64),
    ) -> Result<Self, ()> {
        unsafe {
            // TODO: lifetimes of code sections

            let original_code = get_original_code(memory.as_slice()).unwrap();
            let hook = generate_hook(callback).unwrap();
            let trampoline = generate_trampoline(hook.as_ptr() as u64, &original_code).unwrap();
            let new_code = generate_new_code(trampoline.as_ptr() as u64).unwrap();
            memory.write(&new_code).unwrap();

            Ok(Self {
                memory,
                original_code,
                _hook: hook,
                _trampoline: trampoline,
            })
        }
    }
}

impl Drop for RetHook<'_> {
    fn drop(&mut self) {
        unsafe {
            self.memory.write(&self.original_code).unwrap();
        }
    }
}

fn get_original_code(memory: &[u8]) -> Result<Vec<u8>, IcedError> {
    let new_code_len = generate_new_code(0)?.len();

    let decoder = Decoder::new(BITNESS, memory, DecoderOptions::NONE);

    let mut original_instructions = vec![];
    let mut original_code_len = 0;

    for instruction in decoder {
        original_instructions.push(instruction);
        original_code_len += instruction.len();

        if original_code_len >= new_code_len {
            break;
        }
    }

    let block = InstructionBlock::new(&original_instructions, 0);

    let result = BlockEncoder::encode(BITNESS, block, BlockEncoderOptions::NONE)?;

    let mut code = result.code_buffer;
    code.resize(new_code_len, 0);

    Ok(code)
}

fn generate_hook(callback: extern "win64" fn(u64)) -> Result<ExecutableBuf, IcedError> {
    let mut assembler = CodeAssembler::new(BITNESS)?;

    assembler.push(rax)?;
    assembler.mov(rcx, rax)?;
    assembler.mov(rax, callback as u64)?;
    assembler.call(rax)?;
    assembler.pop(rax)?;

    let instructions = assembler.take_instructions();

    let block = InstructionBlock::new(&instructions, 0);

    let result = BlockEncoder::encode(BITNESS, block, BlockEncoderOptions::NONE)?;

    let executable_buf = ExecutableBuf::new(&result.code_buffer).unwrap();

    Ok(executable_buf)
}

fn generate_trampoline(hook: u64, original_code: &[u8]) -> Result<ExecutableBuf, IcedError> {
    let mut assembler = CodeAssembler::new(BITNESS)?;

    assembler.mov(rax, hook)?;
    assembler.mov(ptr(rsp), rax)?;

    let instructions = assembler.take_instructions();

    let block = InstructionBlock::new(&instructions, 0);

    let result = BlockEncoder::encode(BITNESS, block, BlockEncoderOptions::NONE)?;

    let mut code = result.code_buffer;
    code.extend_from_slice(original_code);

    let executable_buf = ExecutableBuf::new(&code).unwrap();

    Ok(executable_buf)
}

fn generate_new_code(trampoline: u64) -> Result<Vec<u8>, IcedError> {
    let mut assembler = CodeAssembler::new(BITNESS)?;

    assembler.mov(rax, trampoline)?;
    assembler.jmp(rax)?;

    let instructions = assembler.take_instructions();

    let block = InstructionBlock::new(&instructions, 0);

    let result = BlockEncoder::encode(BITNESS, block, BlockEncoderOptions::NONE)?;

    Ok(result.code_buffer)
}
