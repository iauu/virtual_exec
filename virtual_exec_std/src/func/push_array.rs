use virtual_exec_extern::*;
use virtual_exec_type::vm_type::*;
use virtual_exec_core::Machine;
use virtual_exec_type::base::Upcast;

#[fn_extern_wrap]
fn push_array_sync<'a>(machine: &mut Machine<'a>, array: Collection<'a>, item: Any<'a>) -> Result<None, Error> {
    array.write().unwrap().push(item);
    Ok(())
}

extern_link!(PushArray, push_array_sync);
