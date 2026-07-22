use virtual_exec_extern::*;
use virtual_exec_type::vm_type::*;

#[fn_extern_wrap]
fn push_array_sync<'a>( array: Collection<'a>, item: Any<'a>) -> Result<None, Error> {
    array.write_arc_blocking().push(item);
    Ok(())
}

extern_link!(PushArray, push_array_sync, 2);
