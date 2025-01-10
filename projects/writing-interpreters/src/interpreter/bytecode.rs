use std::cell::Cell;

use crate::memory::ArraySize;

use super::{
    array::Array,
    containers::{Container, IndexedContainer, StackAnyContainer, StackContainer},
    error::err_eval,
    list::List,
    safeptr::{MutatorScope, TaggedScopedPtr},
    taggedptr::TaggedPtr,
    CellPtr, MutatorView, RuntimeError, ScopedPtr,
};

/// A register can be in the range 0..255
pub type Register = u8;

/// Literals are stored in a list, a LiteralId describes the index of the value in the list
pub type LiteralId = u16;

type LiteralInteger = i16;

/// Upvalues are stored in a list on a Partial, an UpvalueId is the index into the list
pub type UpvalueId = u8;

/// An instruction jump target is a signed integer, relative to the jump instruction
pub type JumpOffset = i16;
/// Jump offset when the target is still unknown.
pub const JUMP_UNKNOWN: i16 = 0x7fff;

/// Bytecode is stored as fixed-width 32-bit values.
/// This is not the most efficient format but it is easy to work with.
pub type ArrayOpcode = Array<Opcode>;

/// Literals are stored in a separate list of machine-word-width pointers.
/// This is also not the most efficient scheme but it is easy to work with.
pub type Literals = List;

/// Argument count for a function call or partial application
pub type NumArgs = u8;

#[derive(Clone)]
pub struct ByteCode {
    code: ArrayOpcode,
    literals: Literals,
}

impl ByteCode {
    /// Instantiate a blank ByteCode instance
    pub fn alloc<'guard>(
        mem: &'guard MutatorView,
    ) -> Result<ScopedPtr<'guard, ByteCode>, RuntimeError> {
        mem.alloc(ByteCode {
            code: ArrayOpcode::new(),
            literals: Literals::new(),
        })
    }

    /// Append an instuction to the back of the sequence
    pub fn push<'guard>(&self, mem: &'guard MutatorView, op: Opcode) -> Result<(), RuntimeError> {
        self.code.push(mem, op)
    }

    pub fn update_jump_offset<'guard>(
        &self,
        mem: &'guard MutatorView,
        instruction: ArraySize,
        offset: JumpOffset,
    ) -> Result<(), RuntimeError> {
        let code = self.code.get(mem, instruction)?;
        let new_code = match code {
            Opcode::Jump { offset: _ } => Opcode::Jump { offset },
            Opcode::JumpIfTrue { test, offset: _ } => Opcode::JumpIfTrue { test, offset },
            Opcode::JumpIfNotTrue { test, offset: _ } => Opcode::JumpIfNotTrue { test, offset },
            _ => {
                return Err(err_eval(
                    "Cannot modify jump offset for non-jump instruction",
                ))
            }
        };
        self.code.set(mem, instruction, new_code)?;
        Ok(())
    }

    /// Push a literal pointer/value to the back of the literals list and return it's index
    pub fn push_lit<'guard>(
        &self,
        mem: &'guard MutatorView,
        literal: TaggedScopedPtr<'guard>,
    ) -> Result<LiteralId, RuntimeError> {
        let id = self.literals.length() as u16;
        StackAnyContainer::push(&self.literals, mem, literal)?;
        Ok(id)
    }

    pub fn push_loadlit<'guard>(
        &self,
        mem: &'guard MutatorView,
        dest: Register,
        literal_id: LiteralId,
    ) -> Result<(), RuntimeError> {
        self.code.push(
            mem,
            Opcode::LoadLiteral {
                dest,
                literal: literal_id,
            },
        )
    }

    /// Get the index into the bytecode array of the next instruction that will be pushed
    pub fn next_instruction(&self) -> ArraySize {
        self.code.length()
    }

    /// Get the index into the bytecode array of the last instruction
    pub fn last_instruction(&self) -> ArraySize {
        self.code.length() - 1
    }
}

// 4 bytes (1 byte enum tag + 3 bytes of data)
#[derive(Copy, Clone)]
pub enum Opcode {
    Add {
        // 3 bytes
        dest: Register,
        left: Register,
        right: Register,
    },
    Mul {
        // 3 bytes
        dest: Register,
        left: Register,
        right: Register,
    },
    LoadLiteral {
        // 3 bytes
        dest: Register,
        literal: LiteralId,
    },
    Jump {
        offset: JumpOffset,
    },
    JumpIfTrue {
        test: Register,
        offset: JumpOffset,
    },
    JumpIfNotTrue {
        test: Register,
        offset: JumpOffset,
    },
    MakeClosure {
        dest: Register,
        function: Register,
    },
    GetUpvalue {
        dest: Register,
        src: UpvalueId,
    },
    SetUpvalue {
        dest: UpvalueId,
        src: Register,
    },
    CloseUpvalues {
        reg1: Register,
        reg2: Register,
        reg3: Register,
    },
    Return {
        reg: Register,
    },
    LoadNil {
        dest: Register,
    },
    LoadGlobal {
        dest: Register,
        name: Register,
    },
    IsAtom {
        dest: Register,
        test: Register,
    },
    IsNil {
        dest: Register,
        test: Register,
    },
    FirstOfPair {
        dest: Register,
        reg: Register,
    },
    SecondOfPair {
        dest: Register,
        reg: Register,
    },
    MakePair {
        dest: Register,
        reg1: Register,
        reg2: Register,
    },
    IsIdentical {
        dest: Register,
        test1: Register,
        test2: Register,
    },
    StoreGlobal {
        src: Register,
        name: Register,
    },
    CopyRegister {
        dest: Register,
        src: Register,
    },
    Call {
        function: Register,
        dest: Register,
        arg_count: NumArgs,
    },
}

/// An InstructionStream is a pointer to a ByteCode instance and an instruction pointer giving the
/// current index into the ByteCode
pub struct InstructionStream {
    instructions: CellPtr<ByteCode>,
    ip: Cell<ArraySize>,
}

impl InstructionStream {
    /// Create an InstructionStream instance with the given ByteCode instance that will be iterated over
    pub fn alloc<'guard>(
        mem: &'guard MutatorView,
        code: ScopedPtr<'_, ByteCode>,
    ) -> Result<ScopedPtr<'guard, InstructionStream>, RuntimeError> {
        mem.alloc(InstructionStream {
            instructions: CellPtr::new_with(code),
            ip: Cell::new(0),
        })
    }

    pub fn switch_frame(&self, code: ScopedPtr<'_, ByteCode>, ip: ArraySize) {
        self.instructions.set(code);
        self.ip.set(ip);
    }

    /// Retrieve the next instruction and return it, incrementing the instruction pointer
    // TODO: https://github.com/rust-hosted-langs/book/issues/39
    pub fn get_next_opcode<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
    ) -> Result<Opcode, RuntimeError> {
        let instr = self
            .instructions
            .get(guard)
            .code
            .get(guard, self.ip.get())?;
        self.ip.set(self.ip.get() + 1);
        Ok(instr)
    }

    /// Given an index into the literals list, return the pointer in the list at that index.
    pub fn get_literal<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        lit_id: LiteralId,
    ) -> Result<TaggedPtr, RuntimeError> {
        Ok(IndexedContainer::get(
            &self.instructions.get(guard).literals,
            guard,
            lit_id as ArraySize,
        )?
        .get_ptr())
    }

    /// Adjust the instruction pointer by the given signed offset from the current ip
    pub fn jump(&self, offset: JumpOffset) {
        let mut ip = self.ip.get() as i32;
        ip += offset as i32;
        self.ip.set(ip as ArraySize);
    }

    /// Return the next instruction pointer
    pub fn get_next_ip(&self) -> ArraySize {
        self.ip.get()
    }
}

#[cfg(test)]
mod test {
    use super::Opcode;
    use std::mem::size_of;

    #[test]
    fn test_opcode_is_32_bits() {
        // An Opcode should be 32 bits; anything bigger and we've mis-defined some
        // variant
        assert!(size_of::<Opcode>() == 4);
    }
}
