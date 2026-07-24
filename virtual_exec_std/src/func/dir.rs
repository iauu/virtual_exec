use std::alloc::alloc;
use std::sync::{Arc};
use async_lock::RwLock;
use virtual_exec_core::fn_extern::fn_args::FnExternArg::Alloc;
use virtual_exec_extern::*;
use virtual_exec_type::HashMap;
use virtual_exec_type::mem::{Allocator, MemoryAllocator, Value, ValuePtr};
use virtual_exec_type::vm_type::*;

#[fn_extern_wrap]
fn dir_sync<'a>(value: Object<'a>, Alloc(alloc): MemoryAllocator<'a>) -> Result<Collection<'a>, Error> {
    let mem_use = value.read_arc_blocking().iter().map(|(k,_)| k.len()).sum::<usize>() + value.read_arc_blocking().len() * 8;
    alloc.lock_arc_blocking().check_alloc_err(mem_use)?;
    
    let dir: Vec<String> = value.read_arc_blocking().iter().map(|(k, _)| k.clone()).collect();
    Ok(Arc::new(RwLock::new(dir.into_iter().map(|v| alloc.alloc(Value::String(v.into_boxed_str()))).collect::<Result<Vec<_>, _>>()?)))
}

extern_link!(CreateObj, dir_sync, 0);
