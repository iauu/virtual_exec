use virtual_exec_extern::*;
use virtual_exec_type::vm_type::*;
use virtual_exec_core::Machine;
use virtual_exec_type::base::Upcast;
use virtual_exec_type::error::{ExecutionError, NonRecoverableError};

#[fn_extern_wrap]
fn pop_array_sync<'a>(_: &mut Machine<'a>, array: Collection<'a>) -> Result<Any<'a>, Error> {
    array.write_arc_blocking().pop().ok_or(ExecutionError::NonRecoverable(NonRecoverableError::IndexOutOfRangeError))
}

extern_link!(PopArray, pop_array_sync, 1);
