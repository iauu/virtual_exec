use alloc::sync::Arc;
use async_lock::Mutex;
use crate::error::{ExecutionError, NonRecoverableError};
use crate::mem::MemoryAllocator;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecurseConfig {
    pub recurse_depth: Option<u32>,
    pub inst_limit: Option<u64>,
    pub mem_limit: Option<usize>
}

#[derive(Clone)]
pub struct RecurseRestricter<'a> {
    pub config: RecurseConfig,
    pub curr_depth: u32,
    pub curr_inst: Arc<Mutex<u64>>,
    pub curr_mem: Arc<Mutex<usize>>,
    pub alloc: MemoryAllocator<'a>
}

impl Default for RecurseConfig {
    fn default() -> Self {
        Self {
            recurse_depth: Some(256),
            inst_limit: Some(1048576),
            mem_limit: Some(1048576)
        }
    }
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct RecursionError;

impl Into<NonRecoverableError> for RecursionError {
    fn into(self) -> NonRecoverableError {
        NonRecoverableError::RecursionError
    }
}

impl Into<ExecutionError> for RecursionError {
    fn into(self) -> ExecutionError {
        let non_recoverable: NonRecoverableError = self.into();
        non_recoverable.into()
    }
}

impl RecurseConfig {
    pub fn as_lim(self, alloc: MemoryAllocator) -> RecurseRestricter {
        RecurseRestricter {
            config: self,
            curr_depth: 0,
            curr_inst: Arc::new(Mutex::new(0)),
            curr_mem: Arc::new(Mutex::new(0)),
            alloc
        }
    }
}


impl<'a> RecurseRestricter<'a> {
    pub fn incr(&self) -> Result<Self, RecursionError> {
        let mut other = self.clone();
        other.curr_depth += 1;
        if let Some(depth_lim) = other.config.recurse_depth && depth_lim < other.curr_depth {
            return Err(RecursionError);
        }
        Ok(other)
    }

    pub fn consume_inst(&self, amount: u64) -> Result<(), RecursionError> {
        self.curr_inst.lock_arc_blocking().checked_add(amount).ok_or(RecursionError)?;
        if let Some(inst_lim) = self.config.inst_limit && *self.curr_inst.lock_arc_blocking() > inst_lim {
            return Err(RecursionError);
        }
        Ok(())
    }

    pub fn consume_mem(&self, amount: u64) -> Result<(), RecursionError> {
        self.curr_inst.lock_arc_blocking().checked_add(amount).ok_or(RecursionError)?;
        if let Some(inst_lim) = self.config.inst_limit && *self.curr_inst.lock_arc_blocking() > inst_lim {
            return Err(RecursionError);
        }
        self.alloc.lock_arc_blocking().check_alloc_err(*self.curr_mem.lock_arc_blocking()).map_err(|_|RecursionError)?;
        Ok(())
    }
}