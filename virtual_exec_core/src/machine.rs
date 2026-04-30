
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::sequential::exec::{FnStackFrame, InstStateMachine, State};
use crate::sequential::instructions::Instruction;
use virtual_exec_type::error::{ExecutionError, MemoryError};
use virtual_exec_type::mem::{Allocator, MemoryAllocator, MemoryAllocatorConstructor, OwnedValue, ToOwned, Value};
use crate::fn_extern::{FnExtern, MethodResolver};

/// The execution instance including the memory allocator and the instruction state machine
#[derive(Debug)]
pub struct Machine<'a> {
    #[allow(unused)]
    /// The memory allocator for the machine
    pub alloc: MemoryAllocator<'a>,
    /// The instruction execution machine for the instance
    pub machine: InstStateMachine<'a>,
    pub resolvers: Vec<MethodResolver>,
}

impl<'a> Machine<'a> {
    /// Create a new execution instance with the given instructions, memory limit and instruction execution limit
    /// # Arguments
    /// * `instructions` - A vector for the sequential instructions
    /// * `memory_lim` - The amount of memory (in virtual bytes) that can be used by the execution instance
    /// * `inst_limit` - The amount of instruction it can run until it being paused by timeout
    ///
    /// # Returns
    /// `Result<Machine, MemoryError>`
    pub fn new(instructions: Vec<Instruction>, memory_lim: usize, inst_limit: u64, resolvers: Vec<MethodResolver>) -> Result<Self, MemoryError> {
        let alloc = MemoryAllocator::construct(memory_lim);
        let mut map = HashMap::new();
        for resolver in resolvers.iter() {
            for item in resolver.get_pair() {
                let ptr = Value::FnPtrExternal(item.0.clone().into_boxed_str(), item.1);
                let alloced = alloc.alloc(ptr)?;
                map.insert(item.0, alloced);
            }
        }
        let machine = InstStateMachine {
            lim: inst_limit,
            fn_stack_frame: vec![FnStackFrame {
                ptr: 0,
                mapping: Arc::new(RwLock::new(map))
            }],
            alloc: alloc.clone(),
            instructions,
            state: Ok(State::Ok),
            stack: vec![],
        };
        Ok(Self {
            alloc,
            machine,
            resolvers
        })
    }

    pub fn sync_run_once(&mut self) -> Result<(State<'a>, bool), ExecutionError> {
        if let Ok(State::Ok) = self.machine.state {
            self.machine.run_once().map(|x| (x, true))
        }
        else {
            if let Ok(State::FnExternInput(func, _)) = &self.machine.state {
                let fns: Vec<Arc<dyn FnExtern + Send + Sync>> = self.resolvers.iter().filter_map(|x| x.get(func)).collect();
                if fns.len() > 0 {
                    let inputs = self.machine.retrieve_fn_input()?.unwrap();
                    let result = fns[0].fn_extern_sync(self, inputs.1);
                    self.machine.push_fn_output(result);
                    return self.machine.state.clone().map(|x| (x, true))
                }
            }
            self.machine.state.clone().map(|x| (x, false))
        }
    }

    pub fn sync_run_for(&mut self, count: u64) -> Result<State<'a>, ExecutionError> {
        for _ in 0..count {
            if let Ok(State::Ok) | Ok(State::FnExternInput(_, _)) = self.machine.state {
                let result = self.sync_run_once()?;
                if !result.1 {
                    return Ok(result.0)
                }
            }
        }
        self.machine.state.clone()
    }

    pub fn sync_run_all(&mut self) -> Result<State<'a>, ExecutionError> {
        while let Ok(State::Ok) | Ok(State::FnExternInput(_, _)) = self.machine.state {
            let result = self.sync_run_once()?;
            if !result.1 {
                return Ok(result.0)
            }
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
