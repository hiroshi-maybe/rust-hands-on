use std::cell::Cell;

use crate::memory::ArraySize;

use super::{
    array::Array,
    bytecode::{InstructionStream, Opcode},
    containers::{
        Container, HashIndexedAnyContainer, IndexedAnyContainer, IndexedContainer,
        SliceableContainer, StackAnyContainer,
    },
    dict::Dict,
    error::err_eval,
    function::Function,
    list::List,
    safeptr::{MutatorScope, TaggedCellPtr, TaggedScopedPtr},
    taggedptr::{TaggedPtr, Value},
    CellPtr, MutatorView, RuntimeError, ScopedPtr,
};

pub const ENV_REG: usize = 1;
pub const FIRST_ARG_REG: usize = 2;

/// An execution Thread object.
/// It is composed of all the data structures required for execution of a bytecode stream -
/// register stack, call frames, closure upvalues, thread-local global associations and the current
/// instruction pointer.
pub struct Thread {
    /// An array of CallFrames
    frames: CellPtr<CallFrameList>,
    /// The current instruction location
    instr: CellPtr<InstructionStream>,
    /// An array of pointers any object type
    stack: CellPtr<List>,
    /// The current stack base pointer
    stack_base: Cell<ArraySize>,
    /// A dict that should only contain Number keys and Upvalue values. This is a mapping of
    /// absolute stack indeces to Upvalue objects where stack values are closed over.
    upvalues: CellPtr<Dict>,
    /// A dict that should only contain Symbol keys but any type as values
    globals: CellPtr<Dict>,
}

/// Call frames are stored in a separate stack to the register window stack. This simplifies types
/// and stack math.
pub type CallFrameList = Array<CallFrame>;

/// A call frame, separate from the register stack
#[derive(Clone)]
pub struct CallFrame {
    /// Pointer to the Function being executed
    function: CellPtr<Function>,
    /// Return IP when returning from a nested function call
    ip: Cell<ArraySize>,
    /// Stack base - index into the register stack where register window for this function begins
    base: ArraySize,
}

/// Evaluation control flow flags
#[derive(PartialEq)]
pub enum EvalStatus<'guard> {
    /// Eval result is pending, more instructions must be executed
    Pending,
    /// Eval is complete, here is the resulting value
    Return(TaggedScopedPtr<'guard>),
}

/// A closure upvalue as generally described by Lua 5.1 implementation.
/// There is one main difference - in the Lua (and Crafting Interpreters) documentation, an upvalue
/// is closed by pointing the `location` pointer at the `closed` pointer directly in the struct.
/// This isn't a good idea _here_ because a stack location may be invalidated by the stack List
/// object being reallocated. This VM doesn't support pointers into objects.
#[derive(Clone)]
pub struct Upvalue {
    // Upvalue location can't be a pointer because it would be a pointer into the dynamically
    // alloocated stack List - the pointer would be invalidated if the stack gets reallocated.
    value: TaggedCellPtr,
    closed: Cell<bool>,
    location: ArraySize,
}

impl Upvalue {
    /// Allocate a new Upvalue on the heap. The absolute stack index of the object must be
    /// provided.
    fn alloc<'guard>(
        mem: &'guard MutatorView,
        location: ArraySize,
    ) -> Result<ScopedPtr<'guard, Upvalue>, RuntimeError> {
        mem.alloc(Upvalue {
            value: TaggedCellPtr::new_nil(),
            closed: Cell::new(false),
            location,
        })
    }

    /// Dereference the upvalue
    fn get<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        stack: ScopedPtr<'guard, List>,
    ) -> Result<TaggedPtr, RuntimeError> {
        match self.closed.get() {
            true => Ok(self.value.get_ptr()),
            false => Ok(IndexedContainer::get(&*stack, guard, self.location)?.get_ptr()),
        }
    }

    /// Write a new value to the Upvalue, placing it here or on the stack depending on the
    /// closedness of it.
    fn set<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        stack: ScopedPtr<'guard, List>,
        ptr: TaggedPtr,
    ) -> Result<(), RuntimeError> {
        match self.closed.get() {
            true => self.value.set_to_ptr(ptr),
            false => {
                IndexedContainer::set(&*stack, guard, self.location, TaggedCellPtr::new_ptr(ptr))?
            }
        };
        Ok(())
    }

    /// Close the upvalue, copying the stack variable value into the Upvalue
    fn close<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        stack: ScopedPtr<'guard, List>,
    ) -> Result<(), RuntimeError> {
        let ptr = IndexedContainer::get(&*stack, guard, self.location)?.get_ptr();
        self.value.set_to_ptr(ptr);
        self.closed.set(true);
        Ok(())
    }
}

impl Thread {
    /// Execute the next instruction in the current instruction stream
    fn eval_next_instr<'guard>(
        &self,
        mem: &'guard MutatorView,
    ) -> Result<EvalStatus<'guard>, RuntimeError> {
        let frames = self.frames.get(mem);
        let stack = self.stack.get(mem);
        let globals = self.globals.get(mem);
        let instr = self.instr.get(mem);

        // Establish a 256-register window into the stack from the stack base
        stack.access_slice(mem, |full_stack| {
            let stack_base = self.stack_base.get() as usize;
            let window = &mut full_stack[stack_base..stack_base + 256];

            // Fetch the next instruction and identify it
            let opcode = instr.get_next_opcode(mem)?;

            match opcode {
                Opcode::Add { dest, left, right } => {
                    let left = match *window[left as usize].get(mem) {
                        Value::Number(n) => n,
                        _ => todo!(),
                    };
                    let right = match *window[right as usize].get(mem) {
                        Value::Number(n) => n,
                        _ => todo!(),
                    };
                    let result = left + right;
                    let result = TaggedScopedPtr::new(mem, TaggedPtr::number(result));
                    window[dest as usize].set(result);
                }
                // Load a literal into a register from the function literals array
                Opcode::LoadLiteral { dest, literal } => {
                    let literal_ptr = instr.get_literal(mem, literal)?;
                    window[dest as usize].set_to_ptr(literal_ptr);
                }
                // Unconditional jump - advance the instruction pointer by `offset`
                Opcode::Jump { offset } => {
                    instr.jump(offset);
                }
                Opcode::JumpIfTrue { test, offset } => {
                    let test_val = window[test as usize].get(mem);
                    let true_sum = mem.lookup_sym("true"); // TODO preload keyword syms
                    if test_val == true_sum {
                        instr.jump(offset);
                    }
                }
                Opcode::JumpIfNotTrue { test, offset } => {
                    let test_val = window[test as usize].get(mem);
                    let true_sum = mem.lookup_sym("true"); // TODO preload keyword syms
                    if test_val != true_sum {
                        instr.jump(offset);
                    }
                }
                // This operation should be generated by the compiler after a function definition
                // inside another function but only if the nested function refers to nonlocal
                // variables.
                // The result of this operation is a Partial with a closure environment
                Opcode::MakeClosure { dest, function } => {
                    // 1. iter over function nonlocals
                    //   - calculate absolute stack offset for each
                    //   - find existing or create new Upvalue for each
                    //   - create closure environment with list of Upvalues
                    // 2. create new Partial with environment
                    // 3. set dest to Partial
                    let function_ptr = window[function as usize].get(mem);
                    let Value::Function(f) = *function_ptr else {
                        return Err(err_eval("Cannot make a closure from a non-Function type"));
                    };

                    let nonlocal = f.nonlocals(mem);
                    // Create an environment array for upvalues
                    let env = List::alloc_with_capacity(mem, nonlocal.length())?;

                    // Iter over function nonlocals, calculating absolute stack offset for each
                    nonlocal.access_slice(mem, |nonlocals| -> Result<(), RuntimeError> {
                        for compound in nonlocals {
                            // extract 8 bit register and call frame values from 16 bit nonlocal
                            // descriptors
                            let frame_offset = (*compound >> 8) as ArraySize;
                            let window_offset = (*compound & 0xff) as ArraySize;

                            // look back frame_offset frames and add the register number to
                            // calculate the absolute stack position of the value
                            let frame = frames.get(mem, frames.length() - frame_offset)?;
                            let location = frame.base + window_offset;

                            // look up, or create, the Upvalue for the location, and add it to
                            // the environment
                            let (_, upvalue) = self.upvalue_lookup_or_alloc(mem, location)?;
                            StackAnyContainer::push(&*env, mem, upvalue.as_tagged(mem))?;
                        }

                        Ok(())
                    })?;
                }
                Opcode::GetUpvalue { dest, src } => {
                    let closure_env = window[ENV_REG].get(mem);
                    let upvalue = env_upvalue_lookup(mem, closure_env, src)?;
                    window[dest as usize].set_to_ptr(upvalue.get(mem, stack)?);
                }
                Opcode::SetUpvalue { dest, src } => {
                    let closure_env = window[ENV_REG].get(mem);
                    let upvalue = env_upvalue_lookup(mem, closure_env, dest)?;
                    upvalue.set(mem, stack, window[src as usize].get_ptr())?;
                }
                Opcode::CloseUpvalues { reg1, reg2, reg3 } => {
                    for reg in &[reg1, reg2, reg3] {
                        // Registers 0 and 1 cannot be closed over
                        if *reg >= FIRST_ARG_REG as u8 {
                            // calculate absolute stack offset of reg
                            let location = stack_base as ArraySize + *reg as ArraySize;
                            // find the Upvalue object by location
                            let (location_ptr, upvalue) = self.upvalue_lookup(mem, location)?;
                            // close it and unanchor from the Thread
                            upvalue.close(mem, stack)?;
                            self.upvalues.get(mem).dissoc(mem, location_ptr)?;
                        }
                    }
                }
            }

            Ok(EvalStatus::Pending)
        })
    }

    /// Retrieve an Upvalue for the given absolute stack offset or allocate a new one if none was
    /// found
    fn upvalue_lookup_or_alloc<'guard>(
        &self,
        mem: &'guard MutatorView,
        location: ArraySize,
    ) -> Result<(TaggedScopedPtr<'guard>, ScopedPtr<'guard, Upvalue>), RuntimeError> {
        match self.upvalue_lookup(mem, location) {
            Ok(v) => Ok(v),
            Err(_) => {
                let upvalues = self.upvalues.get(mem);
                let upvalue = Upvalue::alloc(mem, location)?;

                let location_ptr = TaggedScopedPtr::new(mem, TaggedPtr::number(location as isize));
                upvalues.assoc(mem, location_ptr, upvalue.as_tagged(mem))?;

                Ok((location_ptr, upvalue))
            }
        }
    }

    /// Retrieve an Upvalue for the given absolute stack offset.
    fn upvalue_lookup<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        location: ArraySize,
    ) -> Result<(TaggedScopedPtr<'guard>, ScopedPtr<'guard, Upvalue>), RuntimeError> {
        let upvalues = self.upvalues.get(guard);

        // Convert the location integer to a TaggedScopedPtr for passing
        // into the Thread's upvalues Dict
        let location_ptr = TaggedScopedPtr::new(guard, TaggedPtr::number(location as isize));

        // Lookup upvalue in upvalues dict
        match upvalues.lookup(guard, location_ptr) {
            Ok(upvalue_ptr) => {
                // Return it and the tagged-pointer version of the location number
                match *upvalue_ptr {
                    Value::Upvalue(upvalue) => Ok((location_ptr, upvalue)),
                    _ => unreachable!(),
                }
            }
            Err(e) => Err(e),
        }
    }
}

/// Get the Upvalue for the index into the given closure environment.
/// Function will panic if types are not as expected.
fn env_upvalue_lookup<'guard>(
    guard: &'guard dyn MutatorScope,
    closure_env: TaggedScopedPtr<'guard>,
    upvalue_id: u8,
) -> Result<ScopedPtr<'guard, Upvalue>, RuntimeError> {
    match *closure_env {
        Value::List(env) => {
            let upvalue_ptr = IndexedAnyContainer::get(&*env, guard, upvalue_id as ArraySize)?;

            match *upvalue_ptr {
                Value::Upvalue(upvalue) => Ok(upvalue),
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}
