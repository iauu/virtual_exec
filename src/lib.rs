use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use virtual_exec_parser::sequential::exec::{FnStackFrame, InstStateMachine, State};
use virtual_exec_parser::sequential::instructions::Instruction;
use virtual_exec_type::error::ExecutionError;
use virtual_exec_type::mem::{MemoryAllocator, MemoryAllocatorConstructor, OwnedValue, ToOwned};
pub use virtual_exec_parser::{parser::parse, sequential::compile::compile};

// The interpreted unified error type for the `virtual_exec` library.
// #[derive(Debug)]
// pub enum InterpretedExecError {
//     /// An error that occurred during the parsing phase.
//     Parse(ParseError),
//     /// An error that occurred during the execution phase.
//     Execution(InterpretedSandboxExecutionError),
// }
//
// impl From<ParseError> for InterpretedExecError {
//     fn from(e: ParseError) -> Self {
//         InterpretedExecError::Parse(e)
//     }
// }
//
// impl From<InterpretedSandboxExecutionError> for InterpretedExecError {
//     fn from(e: InterpretedSandboxExecutionError) -> Self {
//         InterpretedExecError::Execution(e)
//     }
// }

// Executes a string of code in a sandboxed environment.
//
// # Arguments
//
// * `code` - A string slice containing the code to execute.
// * `ttl` - A time-to-live value representing the maximum number of operations allowed.
//
// # Returns
//
// A `Result` which is either:
// * `Ok(HashMap<String, RsValue>)` - A dictionary of the final state of all variables.
// * `Err(InterpretedExecError)` - An error that occurred during parsing or execution.
// pub fn interpreted_exec(code: &str, ttl: i64) -> Result<HashMap<String, RsValue>, InterpretedExecError> {
//     // 1. Parse the code into an AST.
//     let module = parser::parse(code)?;
//
//     let arena = Bump::new();
//     let alloc = Allocator::new(&arena);
//     let global_scope = Rc::new(RefCell::new(Mapping { mapping: HashMap::new() }));
//     let mapping = vec![global_scope];
//     let ctx = Rc::new(RefCell::new(ExecutionContext::new(alloc, ttl, mapping)));
//
//     module.eval(ctx.clone())?;
//
//     let final_state = ctx.borrow().to_hashmap();
//     Ok(final_state)
// }

/// The execution instance including the memory allocator and the instruction state machine
#[derive(Debug)]
pub struct Machine<'a> {
    #[allow(unused)]
    /// The memory allocator for the machine
    pub alloc: MemoryAllocator<'a>,
    /// The instruction execution machine for the instance
    pub machine: InstStateMachine<'a>
}

impl<'a> Machine<'a> {
    /// Create a new execution instance with the given instructions, memory limit and instruction execution limit
    /// # Arguments
    /// * `instructions` - A vector for the sequential instructions
    /// * `memory_lim` - The amount of memory (in virtual bytes) that can be used by the execution instance
    /// * `inst_limit` - The amount of instruction it can run until it being paused by timeout
    ///
    /// # Returns
    /// `Machine`
    pub fn new(instructions: Vec<Instruction>, memory_lim: usize, inst_limit: u64) -> Self {
        let alloc = MemoryAllocator::construct(memory_lim);
        let machine = InstStateMachine {
            lim: inst_limit,
            fn_stack_frame: vec![FnStackFrame {
                ptr: 0,
                mapping: Arc::new(RwLock::new(HashMap::new()))
            }],
            alloc: alloc.clone(),
            instructions,
            state: Ok(State::Ok),
            stack: vec![],
        };
        Self {
            alloc,
            machine
        }
    }

    pub fn run_once(&mut self) -> Result<State<'a>, ExecutionError> {
        self.machine.run_once()
    }

    pub fn run_all(&mut self) -> Result<State<'a>, ExecutionError> {
        while let Ok(State::Ok) = self.machine.state {
            self.machine.run_once()?;
        }
        self.machine.state.clone()
    }
    
    pub fn get(&self, name: &str) -> Option<OwnedValue> {
        for fn_frame in self.machine.fn_stack_frame.iter().rev() {
            if let Some(v) = fn_frame.mapping.read().unwrap().get(name).cloned() {
                return Some(v.lock().unwrap().inner.to_owned_value());
            }
        }
        None
    }
}

