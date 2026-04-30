mod resolver;

use virtual_exec_type::mem::{ValuePtr};
use async_trait::async_trait;
use virtual_exec_type::error::ExecutionError;
use crate::Machine;
pub use crate::fn_extern::resolver::MethodResolver;

pub trait FnExternConstruct {
    fn new() -> Self where Self: Sized;
}

#[async_trait]
pub trait FnExtern : FnExternConstruct {
    fn fn_extern_sync<'a>(&self, machine: &mut Machine<'a>, values: Vec<ValuePtr<'a>>) -> Result<ValuePtr<'a>, ExecutionError>;

    async fn fn_extern_async<'a>(&self, machine: &mut Machine<'a>, values: Vec<ValuePtr<'a>>) -> Result<ValuePtr<'a>, ExecutionError> where
    'a : 'async_trait {
        self.fn_extern_sync(machine, values)
    }
    
    fn get_size(&self) -> usize;
}
