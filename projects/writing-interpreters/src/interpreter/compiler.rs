use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

use crate::memory::ArraySize;

use super::{
    bytecode::{ByteCode, JumpOffset, Opcode, Register, UpvalueId, JUMP_UNKNOWN},
    containers::{AnyContainerFromSlice, StackContainer},
    error::err_eval,
    function::Function,
    list::List,
    pair::{value_from_1_pair, values_from_2_pairs, vec_from_pairs},
    safeptr::TaggedScopedPtr,
    taggedptr::Value,
    vm::FIRST_ARG_REG,
    ArrayU16, CellPtr, MutatorView, RuntimeError, ScopedPtr,
};

/// Compile the given AST and return an anonymous Function object
pub fn compile<'guard>(
    mem: &'guard MutatorView,
    ast: TaggedScopedPtr<'guard>,
) -> Result<ScopedPtr<'guard, Function>, RuntimeError> {
    let compiler = Compiler::new(mem, None)?;
    compiler.compile_function(mem, mem.nil(), &[], &[ast])
}

/// Compile a function - parameters and expression, returning a tagged Function object
fn compile_function<'guard, 'scope>(
    mem: &'guard MutatorView,
    parent: Option<&'scope Variables<'scope>>,
    name: TaggedScopedPtr<'guard>,
    params: &[TaggedScopedPtr<'guard>],
    exprs: &[TaggedScopedPtr<'guard>],
) -> Result<TaggedScopedPtr<'guard>, RuntimeError> {
    let compiler = Compiler::new(mem, parent)?;
    Ok(compiler
        .compile_function(mem, name, params, exprs)?
        .as_tagged(mem))
}

struct Compiler<'parent> {
    bytecode: CellPtr<ByteCode>,
    /// Next available register slot.
    next_reg: Register,
    /// Optional function name
    name: Option<String>,
    /// Function-local nested scopes bindings list (including parameters at outer level)
    vars: Variables<'parent>,
}

/// A variable is a named register. It has compile time metadata about how it is used by closures.
struct Variable {
    register: Register,
    closed_over: Cell<bool>,
}

/// A Scope contains a set of local variable to register bindings
///
/// Manage the mapping of a variable name to the Variable register number within a single scope
struct Scope {
    /// symbol -> variable mapping
    bindings: HashMap<String, Variable>,
}

/// A nonlocal reference will turn in to an Upvalue at VM runtime.
/// This struct stores the non-zero frame offset and register values of a parent function call
/// frame where a binding will be located.
///
/// Cache a relative stack location of a nonlocal variable for compiling upvalues
struct Nonlocal {
    upvalue_id: u8,
    frame_offset: u8,
    frame_register: u8,
}

/// A Variables instance represents a set of nested variable binding scopes for a single function
/// definition.
///
/// Maintain all the nested scopes for a function during compilation and caches all the nonlocal
/// references. It also keeps a reference to a parent nesting function if there is one, in order
/// to handle lexically scoped lookups.
struct Variables<'parent> {
    /// The parent function's variables.
    parent: Option<&'parent Variables<'parent>>,
    /// Nested scopes, starting with parameters/arguments on the outermost scope and let scopes on
    /// the inside.
    scopes: Vec<Scope>,
    /// Mapping of referenced nonlocal nonglobal variables and their upvalue indexes and where to
    /// find them on the stack.
    nonlocals: RefCell<HashMap<String, Nonlocal>>,
    /// The next upvalue index to assign when a new nonlocal is encountered.
    next_upvalue: Cell<u8>,
}

/// A binding can be either local or via an upvalue depending on how a closure refers to it.
#[derive(Copy, Clone, PartialEq)]
enum Binding {
    /// An local variable is local to a function scope
    Local(Register),
    /// An Upvalue is an indirection for pointing at a nonlocal variable on the stack
    Upvalue(UpvalueId),
}

impl<'parent> Compiler<'parent> {
    /// Instantiate a new nested function-level compiler
    fn new<'guard>(
        mem: &'guard MutatorView,
        parent: Option<&'parent Variables<'parent>>,
    ) -> Result<Compiler<'parent>, RuntimeError> {
        Ok(Compiler {
            bytecode: CellPtr::new_with(ByteCode::alloc(mem)?),
            // register 0 is reserved for the return value, 1 is reserved for a closure environment
            next_reg: FIRST_ARG_REG as u8,
            name: None,
            vars: Variables::new(parent),
        })
    }

    /// Compile an expression that has parameters and possibly a name        
    fn compile_function<'guard>(
        mut self,
        mem: &'guard MutatorView,
        name: TaggedScopedPtr<'guard>,
        params: &[TaggedScopedPtr<'guard>],
        exprs: &[TaggedScopedPtr<'guard>],
    ) -> Result<ScopedPtr<'guard, Function>, RuntimeError> {
        // validate function name
        self.name = match *name {
            Value::Symbol(s) => Some(String::from(s.as_str(mem))),
            Value::Nil => None,
            _ => {
                return Err(err_eval(
                    "A function name may be nil (anonymous) or a symbol (named)",
                ))
            }
        };
        let fn_name = name;

        // validate arity
        if params.len() > 254 {
            return Err(err_eval("A function cannot have more than 254 parameters"));
        }
        // put params into a list for the Function object
        let fn_params = List::from_slice(mem, params)?;

        // also assign params to the first level function scope and give each one a register
        let mut param_scope = Scope::new();
        self.next_reg = param_scope.push_bindings(params, self.next_reg)?;
        self.vars.scopes.push(param_scope);

        // validate expression list
        if exprs.len() == 0 {
            return Err(err_eval("A function must have at least one expression"));
        }

        // compile expressions
        let mut result_reg = 0;
        for expr in exprs.iter() {
            result_reg = self.compile_eval(mem, *expr)?;
        }

        // pop parameter scope
        let closing_instructions = self.vars.pop_scope();
        for opcode in &closing_instructions {
            self.push(mem, *opcode)?;
        }

        // finish with a return
        let fn_bytecode = self.bytecode.get(mem);
        fn_bytecode.push(mem, Opcode::Return { reg: result_reg })?;

        let fn_nonlocals = self.vars.get_nonlocals(mem)?;

        Ok(Function::alloc(
            mem,
            fn_name,
            fn_params,
            fn_bytecode,
            fn_nonlocals,
        )?)
    }

    /// Compile an expression - this can be an 'atomic' value or a nested function application    
    fn compile_eval<'guard>(
        &mut self,
        mem: &'guard MutatorView,
        ast_node: TaggedScopedPtr<'guard>,
    ) -> Result<Register, RuntimeError> {
        match *ast_node {
            Value::Pair(p) => self.compile_apply(mem, p.first.get(mem), p.second.get(mem)),
            Value::Symbol(s) => {
                match s.as_str(mem) {
                    "nil" => {
                        let dest = self.acquire_reg();
                        self.push(mem, Opcode::LoadNil { dest })?;
                        Ok(dest)
                    }

                    "true" => self.push_load_literal(mem, mem.lookup_sym("true")),

                    // Search scopes for a binding; if none do a global lookup
                    _ => {
                        match self.vars.lookup_binding(ast_node)? {
                            Some(Binding::Local(register)) => Ok(register),

                            Some(Binding::Upvalue(upvalue_id)) => {
                                // Retrieve the value via Upvalue indirection
                                let dest = self.acquire_reg();
                                self.push(
                                    mem,
                                    Opcode::GetUpvalue {
                                        dest,
                                        src: upvalue_id,
                                    },
                                )?;
                                Ok(dest)
                            }

                            None => {
                                // Otherwise do a late-binding global lookup
                                let name = self.push_load_literal(mem, ast_node)?;
                                let dest = name; // reuse the register
                                self.push(mem, Opcode::LoadGlobal { dest, name })?;
                                Ok(dest)
                            }
                        }
                    }
                }
            }

            _ => self.push_load_literal(mem, ast_node),
        }
    }

    /// Push an instruction to the function bytecode list
    fn push<'guard>(&mut self, mem: &'guard MutatorView, op: Opcode) -> Result<(), RuntimeError> {
        self.bytecode.get(mem).push(mem, op)
    }

    /// Compile a function or special-form application    
    fn compile_apply<'guard>(
        &mut self,
        mem: &'guard MutatorView,
        function: TaggedScopedPtr<'guard>,
        args: TaggedScopedPtr<'guard>,
    ) -> Result<Register, RuntimeError> {
        match *function {
            Value::Symbol(s) => match s.as_str(mem) {
                "quote" => self.push_load_literal(mem, value_from_1_pair(mem, args)?),
                "atom?" => self.push_op2(mem, args, |dest, test| Opcode::IsAtom { dest, test }),
                // ANCHOR: DefCompileApplyIsNil
                "nil?" => self.push_op2(mem, args, |dest, test| Opcode::IsNil { dest, test }),
                // ANCHOR_END: DefCompileApplyIsNil
                "car" => self.push_op2(mem, args, |dest, reg| Opcode::FirstOfPair { dest, reg }),
                "cdr" => self.push_op2(mem, args, |dest, reg| Opcode::SecondOfPair { dest, reg }),
                "cons" => self.push_op3(mem, args, |dest, reg1, reg2| Opcode::MakePair {
                    dest,
                    reg1,
                    reg2,
                }),
                "cond" => self.compile_apply_cond(mem, args),
                "is?" => self.push_op3(mem, args, |dest, test1, test2| Opcode::IsIdentical {
                    dest,
                    test1,
                    test2,
                }),
                "+" => self.push_op3(mem, args, |dest, reg1, reg2| Opcode::Add {
                    dest,
                    left: reg1,
                    right: reg2,
                }),
                "*" => self.push_op3(mem, args, |dest, reg1, reg2| Opcode::Mul {
                    dest,
                    left: reg1,
                    right: reg2,
                }),
                "set" => self.compile_apply_assign(mem, args),
                "def" => self.compile_named_function(mem, args),
                // ANCHOR: DefCompileApplyLambda
                "lambda" => self.compile_anonymous_function(mem, args),
                // ANCHOR_END: DefCompileApplyLambda
                "\\" => self.compile_anonymous_function(mem, args),
                "let" => self.compile_apply_let(mem, args),
                _ => self.compile_apply_call(mem, function, args),
            },

            // Here we allow the value in the function position to be evaluated dynamically
            _ => self.compile_apply_call(mem, function, args),
        }
    }

    // this is a naive way of allocating registers - every result gets it's own register
    fn acquire_reg(&mut self) -> Register {
        // TODO check overflow
        let reg = self.next_reg;
        self.next_reg += 1;
        reg
    }

    // Push a literal onto the literals list and a load instruction onto the bytecode list
    fn push_load_literal<'guard>(
        &mut self,
        mem: &'guard MutatorView,
        literal: TaggedScopedPtr<'guard>,
    ) -> Result<Register, RuntimeError> {
        let result = self.acquire_reg();
        let lit_id = self.bytecode.get(mem).push_lit(mem, literal)?;
        self.bytecode.get(mem).push_loadlit(mem, result, lit_id)?;
        Ok(result)
    }

    /// Push an instruction with a result and a single argument to the function bytecode list    
    fn push_op2<'guard, F>(
        &mut self,
        mem: &'guard MutatorView,
        params: TaggedScopedPtr<'guard>,
        f: F,
    ) -> Result<Register, RuntimeError>
    where
        F: Fn(Register, Register) -> Opcode,
    {
        let result = self.acquire_reg();
        let reg1 = self.compile_eval(mem, value_from_1_pair(mem, params)?)?;
        self.bytecode.get(mem).push(mem, f(result, reg1))?;
        Ok(result)
    }

    /// Push an instruction with a result and two arguments to the function bytecode list
    fn push_op3<'guard, F>(
        &mut self,
        mem: &'guard MutatorView,
        params: TaggedScopedPtr<'guard>,
        f: F,
    ) -> Result<Register, RuntimeError>
    where
        F: Fn(Register, Register, Register) -> Opcode,
    {
        let result = self.acquire_reg();
        let (first, second) = values_from_2_pairs(mem, params)?;
        let reg1 = self.compile_eval(mem, first)?;
        let reg2 = self.compile_eval(mem, second)?;
        self.bytecode.get(mem).push(mem, f(result, reg1, reg2))?;
        Ok(result)
    }

    /// Compile a 'cond' application
    /// (cond
    ///   (<if-expr-is-true?>) (<then-expr>)
    ///   (<or-expr-is-true?) (<then-expr>)
    /// )
    /// result is nil if no expression evaluates to true
    fn compile_apply_cond<'guard>(
        &mut self,
        mem: &'guard MutatorView,
        args: TaggedScopedPtr<'guard>,
    ) -> Result<Register, RuntimeError> {
        //
        //   for each arg:
        //     eval cond
        //     if false then jmp -> next
        //     else eval expr
        //     jmp -> end
        //
        let bytecode = self.bytecode.get(mem);

        let mut end_jumps: Vec<ArraySize> = Vec::new();
        let mut last_cond_jump: Option<ArraySize> = None;

        let dest = self.next_reg;

        let mut head = args;
        while let Value::Pair(p) = *head {
            let cond = p.first.get(mem);
            head = p.second.get(mem);
            match *head {
                Value::Pair(p) => {
                    let expr = p.first.get(mem);
                    head = p.second.get(mem);

                    // if this is not the first condition, set the offset of the last
                    // condition-not-true jump to the beginning of this condition
                    if let Some(address) = last_cond_jump {
                        let offset = bytecode.next_instruction() - address - 1;
                        bytecode.update_jump_offset(mem, address, offset as JumpOffset)?;
                    }

                    // We have a condition to evaluate. If the resut is Not True, jump to the
                    // next condition.
                    self.reset_reg(dest); // reuse this register for condition and dest
                    let test: u8 = self.compile_eval(mem, cond)?;
                    let offset = JUMP_UNKNOWN;
                    self.push(mem, Opcode::JumpIfNotTrue { test, offset })?;
                    last_cond_jump = Some(bytecode.last_instruction());

                    // Compile the expression and jump to the end of the entire cond
                    self.reset_reg(dest); // reuse this register for condition and dest
                    let _expr_result = self.compile_eval(mem, expr)?;
                    let offset = JUMP_UNKNOWN;
                    bytecode.push(mem, Opcode::Jump { offset })?;
                    end_jumps.push(bytecode.last_instruction());
                }

                _ => return Err(err_eval("Unexpected end of cond list")),
            }
        }

        // Close out with a default nil result if none of the conditions passed
        if let Some(address) = last_cond_jump {
            self.reset_reg(dest);
            self.push(mem, Opcode::LoadNil { dest })?;
            let offset = bytecode.next_instruction() - address - 1;
            bytecode.update_jump_offset(mem, address, offset as JumpOffset)?;
        }

        // Update all the post-expr jumps to point at the next instruction after the entire cond
        for address in end_jumps.iter() {
            let offset = bytecode.next_instruction() - address - 1;
            bytecode.update_jump_offset(mem, *address, offset as JumpOffset)?;
        }

        Ok(dest)
    }

    // reset the next register back to the given one so that it is reused
    fn reset_reg(&mut self, reg: Register) {
        self.next_reg = reg
    }

    /// Assignment expression - evaluate the two expressions, binding the result of the first
    /// to the (hopefully) symbol provided by the second
    /// (set <identifier-expr> <expr>)
    fn compile_apply_assign<'guard>(
        &mut self,
        mem: &'guard MutatorView,
        params: TaggedScopedPtr<'guard>,
    ) -> Result<Register, RuntimeError> {
        let (first, second) = values_from_2_pairs(mem, params)?;
        let src = self.compile_eval(mem, second)?;
        let name = self.compile_eval(mem, first)?;
        self.push(mem, Opcode::StoreGlobal { src, name })?;
        Ok(src)
    }

    /// (def name (args) (expr))
    fn compile_named_function<'guard>(
        &mut self,
        mem: &'guard MutatorView,
        params: TaggedScopedPtr<'guard>,
    ) -> Result<Register, RuntimeError> {
        let items = vec_from_pairs(mem, params)?;

        if items.len() < 3 {
            return Err(err_eval(
                "A function definition must have at least (def name (params) expr)",
            ));
        }

        // a function consists of (name (params) expr1 .. exprn)
        let fn_name = items[0];
        let fn_params = vec_from_pairs(mem, items[1])?;
        let fn_exprs = &items[2..];

        // compile the function to a Function object
        let fn_object = compile_function(mem, Some(&self.vars), fn_name, &fn_params, fn_exprs)?;

        // load the function object as a literal and associate it with a global name
        // TODO store in local scope if we're nested in an expression
        let name = self.push_load_literal(mem, fn_name)?;
        let src = self.push_load_literal(mem, fn_object)?;
        self.push(mem, Opcode::StoreGlobal { src, name })?;

        Ok(src)

        // TODO if fn_object has nonlocal refs, compile a MakeClosure instruction in addition
    }

    /// (lambda (args) (exprs))
    /// OR
    /// (\ (args) (exprs))        
    fn compile_anonymous_function<'guard>(
        &mut self,
        mem: &'guard MutatorView,
        params: TaggedScopedPtr<'guard>,
    ) -> Result<Register, RuntimeError> {
        let items = vec_from_pairs(mem, params)?;

        if items.len() < 2 {
            return Err(err_eval(
                "An anonymous function definition must have at least (lambda (params) expr)",
            ));
        }

        // a function consists of (name (params) expr1 .. exprn)
        let fn_params = vec_from_pairs(mem, items[0])?;
        let fn_exprs = &items[1..];

        // compile the function to a Function object
        let fn_object = compile_function(mem, Some(&self.vars), mem.nil(), &fn_params, fn_exprs)?;

        // load the function object as a literal
        let dest = self.push_load_literal(mem, fn_object)?;

        // if fn_object has nonlocal refs, compile a MakeClosure instruction in addition, replacing
        // the Function register with a Partial with a closure environment
        match *fn_object {
            Value::Function(f) => {
                if f.is_closure() {
                    self.push(
                        mem,
                        Opcode::MakeClosure {
                            function: dest,
                            dest,
                        },
                    )?;
                }
            }
            // 's gotta be a function
            _ => unreachable!(),
        }

        Ok(dest)
    }

    /// Basic non-recursive let expressions
    /// (let
    ///   ((<name> <expr>)
    ///    (<name> <expr>))
    ///   (<expr>)
    /// )
    fn compile_apply_let<'guard>(
        &mut self,
        mem: &'guard MutatorView,
        args: TaggedScopedPtr<'guard>,
    ) -> Result<Register, RuntimeError> {
        let let_expr = vec_from_pairs(mem, args)?;
        if let_expr.len() < 2 {
            return Err(err_eval("A let expression must have at least 2 arguments"));
        }

        // the binding expressions should be a pair-list itself, and each expression another
        // pair list of length 2.  Convert it to a Vec<(name, expr)> structure for convenience.
        let let_exprs: Vec<(TaggedScopedPtr<'guard>, TaggedScopedPtr<'guard>)> = {
            let vec_of_pairs = vec_from_pairs(mem, let_expr[0])?;
            let mut vec_of_tuples = Vec::new();
            for pairs in &vec_of_pairs {
                vec_of_tuples.push(values_from_2_pairs(mem, *pairs)?);
            }
            vec_of_tuples
        };

        // acquire a let expression dest reg
        let dest = self.acquire_reg();

        // get the names of each binding to push a scope, assigning registers post-result for
        // each binding
        let names: Vec<TaggedScopedPtr<'guard>> = let_exprs.iter().map(|tup| tup.0).collect();

        let mut let_scope = Scope::new();
        self.next_reg = let_scope.push_bindings(&names, self.next_reg)?;
        self.vars.scopes.push(let_scope);

        // compile each binding expression
        for (name, expr) in let_exprs {
            let src = self.compile_eval(mem, expr)?;
            let dest = self.compile_eval(mem, name)?;
            // TODO - more efficient to be able to write the result directly to the let binding reg
            self.push(mem, Opcode::CopyRegister { dest, src })?;
        }

        // compile the expressions after the bindings
        let result_exprs = &let_expr[1..];

        for expr in result_exprs {
            let src = self.compile_eval(mem, *expr)?;
            // TODO - more efficient to be able to write the result directly to the let binding reg
            self.push(mem, Opcode::CopyRegister { dest, src })?;
        }

        // finish up - pop the scope, de-scope all registers except the result, return the result
        let closing_instructions = self.vars.pop_scope();
        for opcode in &closing_instructions {
            self.push(mem, *opcode)?;
        }

        self.reset_reg(dest + 1);
        Ok(dest)
    }

    /// (name <arg-expr-1> <arg-expr-n>)
    fn compile_apply_call<'guard>(
        &mut self,
        mem: &'guard MutatorView,
        function_expr: TaggedScopedPtr<'guard>,
        args: TaggedScopedPtr<'guard>,
    ) -> Result<Register, RuntimeError> {
        // allocate a register for the return value
        let dest = self.acquire_reg();
        // allocate a register for a closure environment pointer
        let _closure_env = self.acquire_reg();

        // evaluate arguments first
        let arg_list = vec_from_pairs(mem, args)?;
        let arg_count = arg_list.len() as u8;

        for arg in arg_list {
            let src = self.compile_eval(mem, arg)?;
            // if a local variable register was returned, we need to copy the register to the arg
            // list. Bound registers are necessarily lower indexes than where the function call is
            // situated because expression scope and register acquisition progresses the register
            // index in use.
            if src <= dest {
                let dest = self.acquire_reg();
                self.push(mem, Opcode::CopyRegister { dest, src })?;
            }
        }

        // put the function pointer in the last register of the call so it'll be discarded
        let function = self.compile_eval(mem, function_expr)?;
        self.push(
            mem,
            Opcode::Call {
                function,
                dest,
                arg_count,
            },
        )?;

        // ignore use of any registers beyond the result once the call is complete
        self.reset_reg(dest + 1);
        Ok(dest)
    }
}

impl<'parent> Variables<'parent> {
    fn new(parent: Option<&'parent Variables<'parent>>) -> Variables<'parent> {
        Variables {
            parent,
            scopes: Vec::new(),
            nonlocals: RefCell::new(HashMap::new()),
            next_upvalue: Cell::new(0),
        }
    }

    /// Pop the last scoped variables and create close-upvalue instructions for any closed over
    fn pop_scope<'guard>(&mut self) -> Vec<Opcode> {
        let mut closings = Vec::new();

        if let Some(scope) = self.scopes.pop() {
            for var in scope.bindings.values() {
                if var.is_closed_over() {
                    closings.push(Opcode::CloseUpvalues {
                        reg1: var.register(),
                        // TODO we can close up to 3 upvalues per opcode
                        reg2: 0,
                        reg3: 0,
                    });
                }
            }
        }

        closings
    }

    /// Return an ArrayU16 of nonlocal references if there are any for the function
    fn get_nonlocals<'guard>(
        &self,
        mem: &'guard MutatorView,
    ) -> Result<Option<ScopedPtr<'guard, ArrayU16>>, RuntimeError> {
        let count = self.next_upvalue.get();
        if count == 0 {
            Ok(None)
        } else {
            let nonlocals = self.nonlocals.borrow();
            let mut values: Vec<_> = nonlocals.values().collect();
            values.sort_by(|x, y| x.upvalue_id.cmp(&y.upvalue_id));

            let list = ArrayU16::alloc_with_capacity(mem, count as ArraySize)?;

            for value in &values {
                let compound: u16 = (value.frame_offset as u16) << 8 | value.frame_register as u16;
                list.push(mem, compound)?;
            }

            Ok(Some(list))
        }
    }

    /// Search for a binding, following parent scopes.
    fn lookup_binding<'guard>(
        &self,
        name: TaggedScopedPtr<'guard>,
    ) -> Result<Option<Binding>, RuntimeError> {
        //  return value should be (count-of-parent-functions-followed, Variable)
        let name_string = match *name {
            Value::Symbol(s) => String::from(s.as_str(&name)),
            _ => {
                return Err(err_eval(
                    "Cannot lookup a variable bound to a non-symbol type",
                ))
            }
        };

        // The frame_offset is the number of parent nesting functions searched for a variable
        let mut frame_offset: u8 = 0;

        let mut locals = Some(self);
        while let Some(l) = locals {
            for scope in l.scopes.iter().rev() {
                if let Some(var) = scope.lookup_binding(&name_string) {
                    if frame_offset == 0 {
                        // At depth 0, this is a local binding
                        return Ok(Some(Binding::Local(var.register())));
                    } else {
                        // Otherwise it is a nonlocal and needs to be referenced as an upvalue.
                        // Create a new upvalue reference if one does not exist.
                        let mut nonlocals = self.nonlocals.borrow_mut();

                        if let None = nonlocals.get(&name_string) {
                            // Create a new non-local descriptor and add it
                            let nonlocal = Nonlocal::new(
                                self.acquire_upvalue_id(),
                                frame_offset,
                                var.register(),
                            );
                            nonlocals.insert(name_string.clone(), nonlocal);

                            // Mark the variable as closed-over, as in, a closure will refer to it
                            // and it's upvalue must be closed at runtime
                            var.close_over();
                        }
                    }
                }
            }

            locals = l.parent;
            frame_offset += 1;
        }

        // We've reached the end of the scopes at this point so we can check if we
        // know about this binding as an upvalue and return it
        let nonlocals = self.nonlocals.borrow();
        if let Some(nonlocal) = nonlocals.get(&name_string) {
            return Ok(Some(Binding::Upvalue(nonlocal.upvalue_id)));
        }

        Ok(None)
    }

    /// Return the next upvalue id and increment the counter
    fn acquire_upvalue_id(&self) -> UpvalueId {
        let id = self.next_upvalue.get();
        self.next_upvalue.set(id + 1);
        id
    }
}

impl Scope {
    fn new() -> Scope {
        Scope {
            bindings: HashMap::new(),
        }
    }

    /// Push a block of bindings into this scope, returning the next register available
    /// after these bound registers. All these variables will be Unclosed by default.
    fn push_bindings<'guard>(
        &mut self,
        names: &[TaggedScopedPtr<'guard>],
        start_reg: Register,
    ) -> Result<Register, RuntimeError> {
        let mut reg = start_reg;
        for name in names {
            self.push_binding(*name, reg)?;
            reg += 1;
        }
        Ok(reg)
    }

    /// Add a Symbol->Register binding to this scope
    fn push_binding<'guard>(
        &mut self,
        name: TaggedScopedPtr<'guard>,
        reg: Register,
    ) -> Result<(), RuntimeError> {
        let name_string = match *name {
            Value::Symbol(s) => String::from(s.as_str(&name)),
            _ => return Err(err_eval("A binding name must be a symbol")),
        };

        self.bindings.insert(name_string, Variable::new(reg));

        Ok(())
    }

    /// Find a Symbol->Register binding in this scope
    fn lookup_binding<'guard>(&self, name: &str) -> Option<&Variable> {
        self.bindings.get(name)
    }
}

impl Variable {
    fn new(register: Register) -> Variable {
        Variable {
            register,
            closed_over: Cell::new(false),
        }
    }

    fn register(&self) -> Register {
        self.register
    }

    fn is_closed_over(&self) -> bool {
        self.closed_over.get()
    }

    fn close_over(&self) {
        self.closed_over.set(true);
    }
}

impl Nonlocal {
    fn new(upvalue_id: UpvalueId, frame_offset: u8, frame_register: Register) -> Nonlocal {
        Nonlocal {
            upvalue_id,
            frame_offset,
            frame_register,
        }
    }
}

/// INTEGRATION TESTS
/// TODO - move to a separate module
#[cfg(test)]
mod integration {
    use super::*;
    use crate::interpreter::memory::Memory;
    use crate::interpreter::parser::parse;
    use crate::interpreter::vm::Thread;
    use crate::interpreter::Mutator;

    fn eval_helper<'guard>(
        mem: &'guard MutatorView,
        thread: ScopedPtr<'guard, Thread>,
        code: &str,
    ) -> Result<TaggedScopedPtr<'guard>, RuntimeError> {
        let compiled_code = compile(mem, parse(mem, code)?)?;
        println!("RUN CODE {}", code);
        let result = thread.quick_vm_eval(mem, compiled_code)?;
        println!("RUN RESULT {}", result);
        Ok(result)
    }

    fn test_helper(test_fn: fn(&MutatorView) -> Result<(), RuntimeError>) {
        let mem = Memory::new();

        struct Test {}
        impl Mutator for Test {
            type Input = fn(&MutatorView) -> Result<(), RuntimeError>;
            type Output = ();

            fn run(
                &self,
                mem: &MutatorView,
                test_fn: Self::Input,
            ) -> Result<Self::Output, RuntimeError> {
                test_fn(mem)
            }
        }

        let test = Test {};
        mem.mutate(&test, test_fn).unwrap();
    }

    #[test]
    fn compile_add_function() {
        fn test_inner(mem: &MutatorView) -> Result<(), RuntimeError> {
            // this test calls a function from another function
            let code = "(+ 1 2)";

            let t = Thread::alloc(mem)?;
            let result = eval_helper(mem, t, code)?;

            assert!(result == mem.number(3));

            Ok(())
        }

        test_helper(test_inner);
    }

    #[test]
    fn compile_recursive_function() {
        fn test_inner(mem: &MutatorView) -> Result<(), RuntimeError> {
            // this test calls a function from another function
            let def_fn = "(def sum (n) (cond (is? n 0) 0 true (+ n (sum (+ n -1)))))";
            let query = "(sum 3)";
            // (sum 3) = 3 + (sum 2) = 3 + (2 + (sum 1)) = 3 + (2 + (1 + (sum 0))) = 3 + (2 + (1 + 0)) = 6

            let t = Thread::alloc(mem)?;
            eval_helper(mem, t, def_fn)?;
            let result = eval_helper(mem, t, query)?;

            assert!(result == mem.number(6));

            Ok(())
        }

        test_helper(test_inner);
    }

    #[test]
    fn compile_mul_function() {
        fn test_inner(mem: &MutatorView) -> Result<(), RuntimeError> {
            // this test calls a function from another function
            let code = "(* 3 4)";

            let t = Thread::alloc(mem)?;
            let result = eval_helper(mem, t, code)?;

            assert!(result == mem.number(12));

            Ok(())
        }

        test_helper(test_inner);
    }

    #[test]
    fn compile_recursive_factorial() {
        fn test_inner(mem: &MutatorView) -> Result<(), RuntimeError> {
            // this test calls a function from another function
            let def_fn = "(def fact (n) (cond (is? n 0) 1 true (* n (fact (+ n -1)))))";
            let query = "(fact 4)";
            // (fact 4) = 4 * (fact 3) = 4 * (3 * (fact 2)) = 4 * (3 * (2 * (fact 1))) = 4 * (3 * (2 * (1 * (fact 0)))) = 4 * (3 * (2 * (1 * 1))) = 24

            let t = Thread::alloc(mem)?;
            eval_helper(mem, t, def_fn)?;
            let result = eval_helper(mem, t, query)?;

            assert!(result == mem.number(24));

            Ok(())
        }

        test_helper(test_inner);
    }

    #[test]
    fn compile_let_expression() {
        fn test_inner(mem: &MutatorView) -> Result<(), RuntimeError> {
            // this test calls a function from another function
            let code = "(let ((x 1) (y 2)) (+ x y))";

            let t = Thread::alloc(mem)?;
            let result = eval_helper(mem, t, code)?;

            assert!(result == mem.number(3));

            Ok(())
        }

        test_helper(test_inner);
    }

    #[test]
    fn compile_let_lambda_expression() {
        fn test_inner(mem: &MutatorView) -> Result<(), RuntimeError> {
            // this test calls a function from another function
            let code = "(let ((f (lambda () 1))) (f))";

            let t = Thread::alloc(mem)?;
            let result = eval_helper(mem, t, code)?;

            assert!(result == mem.number(1));

            Ok(())
        }

        test_helper(test_inner);
    }

    #[test]
    fn compile_let_lambda_arg_expression() {
        fn test_inner(mem: &MutatorView) -> Result<(), RuntimeError> {
            // this test calls a function from another function
            let code = "(let ((f (lambda (x) (+ x 1)))) (f 2))";

            let t = Thread::alloc(mem)?;
            let result = eval_helper(mem, t, code)?;

            assert!(result == mem.number(3));

            Ok(())
        }

        test_helper(test_inner);
    }

    #[test]
    fn compile_let_lambda_arg_2_expression() {
        fn test_inner(mem: &MutatorView) -> Result<(), RuntimeError> {
            // this test calls a function from another function
            let code = "(let ((f (lambda (x y) (+ x y)))) (f 2 3))";

            let t = Thread::alloc(mem)?;
            let result = eval_helper(mem, t, code)?;

            assert!(result == mem.number(5));

            Ok(())
        }

        test_helper(test_inner);
    }

    #[test]
    fn compile_inner_let_lambda_arg_2_expression() {
        fn test_inner(mem: &MutatorView) -> Result<(), RuntimeError> {
            let def_fn = "(def fact (a) (let ((f (lambda (x y) (+ x y)))) (f 2 a)))";

            let query = "(fact 3)";

            let t = Thread::alloc(mem)?;
            eval_helper(mem, t, def_fn)?;
            let result = eval_helper(mem, t, query)?;

            assert!(result == mem.number(5));

            Ok(())
        }

        test_helper(test_inner);
    }
}
