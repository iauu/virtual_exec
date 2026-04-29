use virtual_exec_type::mem::{OwnedValue, ValuePtr};
use async_trait::async_trait;
use virtual_exec_type::error::ExecutionError;
use crate::Machine;

pub trait FnExternConstruct {
    fn new() -> Self;
}

#[async_trait]
pub trait FnExtern<'a> : FnExternConstruct {
    fn fn_extern_sync(&self, machine: &mut Machine<'a>, values: Vec<ValuePtr<'a>>) -> Result<ValuePtr<'a>, ExecutionError>;

    async fn fn_extern_async(&self, machine: &mut Machine<'a>, values: Vec<ValuePtr<'a>>) -> Result<ValuePtr<'a>, ExecutionError> where
    'a : 'async_trait {
        self.fn_extern_sync(machine, values)
    }
}

// Example?
pub mod push_array {
    use std::marker::PhantomData;
    use std::sync::{Arc, RwLock};
    use virtual_exec_type::base::{Downcast};
    use virtual_exec_type::error::ExecutionError;
    use virtual_exec_type::mem::{Allocator, Value, ValuePtr};
    use crate::fn_extern::{FnExtern, FnExternConstruct};
    use crate::Machine;


    pub struct push_array<'a> {
        _scope: PhantomData<&'a ()>,
    }

    impl<'a> FnExternConstruct for push_array<'a> {
        fn new() -> Self {
            Self {
                _scope: PhantomData,
            }
        }
    }

    impl<'a> FnExtern<'a> for push_array<'a> {

        fn fn_extern_sync(&self, machine: &mut Machine<'a>, values: Vec<ValuePtr<'a>>) -> Result<ValuePtr<'a>, ExecutionError> {
            let arg_0 = Downcast::from_value(values[0].clone()).ok_or(ExecutionError::InvalidTypeError)?;
            let arg_1 = Downcast::from_value(values[1].clone()).ok_or(ExecutionError::InvalidTypeError)?;
            fn push_array<'a>(machine: &mut Machine<'a>, array: Arc<RwLock<Vec<ValuePtr<'a>>>>, value: ValuePtr<'a>) -> Result<ValuePtr<'a>, ExecutionError> {
                array.write().unwrap().push(value);
                machine.alloc.alloc(Value::None).or_else(|e| Err(e.into()))
            }
            let result = push_array(machine, arg_0, arg_1)?;
            for mut item in values {
                machine.alloc.change_alloc(&mut item);
            }
            Ok(result)
        }
    }
}