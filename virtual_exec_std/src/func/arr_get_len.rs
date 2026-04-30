use virtual_exec_extern::*;
use virtual_exec_type::vm_type::*;
use virtual_exec_core::Machine;

#[fn_extern_wrap]
fn arr_get_len<'a>(machine: &mut Machine<'a>, array: Collection<'a>) -> Result<Integer, Error> {
    Ok(array.read().unwrap().len() as i64)
}

extern_link!(ArrGetLen, arr_get_len, 1);
