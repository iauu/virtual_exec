use virtual_exec_type::vm_type::{Collection, Any, Error};
use virtual_exec_core::Machine;
use virtual_exec_macro::fn_extern_wrap;
use virtual_exec_type::mem::{Allocator, Value, ValuePtr};
use crate::extern_link;

#[fn_extern_wrap]
fn push_array_sync<'a>(machine: &mut Machine<'a>, collection: Collection<'a>, item: Any<'a>) -> Result<Any<'a>, Error> {
    collection.write().unwrap().push(ValuePtr::from(item));
    machine.alloc.alloc(Value::None).or(Err(Error::MemoryError))
}

extern_link!(PushArray, push_array_sync);