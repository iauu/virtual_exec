use std::sync::{Arc};
use async_lock::RwLock;
use virtual_exec_extern::*;
use virtual_exec_type::vm_type::*;
use virtual_exec_core::Machine;

#[fn_extern_wrap]
fn create_array_sync<'a>(_: &mut Machine<'a>) -> Result<Collection<'a>, Error> {
    Ok(Arc::new(RwLock::new(Vec::new())))
}

extern_link!(CreateArray, create_array_sync, 0);
