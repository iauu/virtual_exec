
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::sequential::exec::{FnStackFrame, InstStateMachine, State};
use crate::sequential::instructions::Instruction;
use virtual_exec_type::error::ExecutionError;
use virtual_exec_type::mem::{MemoryAllocator, MemoryAllocatorConstructor, OwnedValue, ToOwned};

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

    pub fn run_for(&mut self, count: u64) -> Result<State<'a>, ExecutionError> {
        for _ in 0..count {
            if let Ok(State::Ok) = self.machine.state {
                self.machine.run_once()?;
            } else {
                return self.machine.state.clone();
            }
        }
        self.machine.state.clone()
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
