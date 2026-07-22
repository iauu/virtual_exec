use virtual_exec_extern::*;
use virtual_exec_type::vm_type::*;
use virtual_exec_type::error::{ExecutionError, NonRecoverableError};
use virtual_exec_type::ext::SafeReadArcExt;

#[fn_extern_wrap]
fn pop_array_sync<'a>(array: Collection<'a>) -> Result<Any<'a>, Error> {
    let capacity = array.read_arc_safe().capacity();
    let length = array.read_arc_safe().len();
    if capacity.saturating_sub(100) > length || capacity / length.max(1) > 2 {
        array.write_arc_blocking().shrink_to_fit();
    }
    array.write_arc_blocking().pop().ok_or(ExecutionError::NonRecoverable(NonRecoverableError::IndexOutOfRangeError))
}

extern_link!(PopArray, pop_array_sync, 1);
