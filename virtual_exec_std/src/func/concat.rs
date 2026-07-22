use virtual_exec_core::fn_extern::fn_args::FnExternArg::Alloc;
use virtual_exec_extern::*;
use virtual_exec_type::vm_type::*;
use virtual_exec_type::mem::MemoryAllocator;

#[fn_extern_wrap]
fn concat<'a>(left: Str, right: Str, Alloc(alloc): MemoryAllocator) -> Result<Str, Error> {
    alloc.lock_arc_blocking().check_alloc_err(left.len() + right.len())?;
    let mut s = String::with_capacity(left.len() + right.len());
    s.push_str(&left);
    s.push_str(&right);
    Ok(s)
}

extern_link!(Concat, concat, 2);
