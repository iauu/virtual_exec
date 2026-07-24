use std::sync::{Arc};
use async_lock::RwLock;
use virtual_exec_extern::*;
use virtual_exec_type::HashMap;
use virtual_exec_type::mem::ValuePtr;
use virtual_exec_type::vm_type::*;

#[fn_extern_wrap]
fn create_obj_sync<'a>() -> Result<Object<'a>, Error> {
    Ok(Arc::new(RwLock::new(HashMap::<String, ValuePtr<'a>>::new())))
}

extern_link!(CreateObj, create_obj_sync, 0);
