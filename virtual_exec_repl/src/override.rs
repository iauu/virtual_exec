use std::sync::{LazyLock, Mutex};
use virtual_exec_core::fn_extern::fn_args::FnExternArg::Recurse;
use virtual_exec_core::fn_extern::MethodResolver;
use virtual_exec_extern::*;
use virtual_exec_type::vm_type::*;
use virtual_exec_type::base::{ToStringSafe, TypeCast};
use virtual_exec_type::ext::*;

pub static PRINT_BUFFER: Mutex<String> = Mutex::new(String::new());

#[fn_extern_wrap]
fn print<'a>(str: Any<'a>, Recurse(recurse): _) -> Result<None, Error> {
    if let Some(s) = str.as_string() {
        PRINT_BUFFER.lock().unwrap().push_str(&format!("{}", s));
    } else {
        PRINT_BUFFER.lock().unwrap().push_str(&str.read_arc_blocking().to_string_safe(recurse).map_err(|e| into!(e, Error))?);
    }
    Ok(())
}

extern_link!(Print, print, 1);

#[fn_extern_wrap]
fn println<'a>(str: Any<'a>, Recurse(recurse): _) -> Result<None, Error> {
    if let Some(s) = str.as_string() {
        PRINT_BUFFER.lock().unwrap().push_str(&format!("{}\n", s));
    } else {
        PRINT_BUFFER.lock().unwrap().push_str(&str.read_arc_blocking().to_string_safe(recurse).map_err(|e| into!(e, Error))?);
        PRINT_BUFFER.lock().unwrap().push_str("\n");
    }
    Ok(())
}

extern_link!(PrintLn, println, 1);

#[fn_extern_wrap]
fn is_none<'a>(obj: Any<'a>) -> Result<Boolean, Error> {
    Ok(obj.as_none().is_some())
}

extern_link!(IsNone, is_none, 1);

pub static OVERRIDE: LazyLock<MethodResolver> = LazyLock::new(||
    resolve!(
        ("print", Print),
        ("println", PrintLn),
        ("is_none", IsNone)
    )
);