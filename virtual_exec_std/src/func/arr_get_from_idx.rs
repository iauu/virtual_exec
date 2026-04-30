use virtual_exec_extern::*;
use virtual_exec_type::vm_type::*;
use virtual_exec_core::Machine;
use virtual_exec_type::error::ExecutionError;

#[fn_extern_wrap]
fn arr_get_from_idx<'a>(machine: &mut Machine<'a>, array: Collection<'a>, idx: Integer) -> Result<Any<'a>, Error> {
    array.write().unwrap().get(idx as usize).ok_or(ExecutionError::IndexOutOfRangeError).map(|x| x.clone())
}

extern_link!(ArrGetFromIdx, arr_get_from_idx, 2);
