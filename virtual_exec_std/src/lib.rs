pub mod func;

#[cfg(feature = "sys")]
pub mod sys;

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use virtual_exec_core::fn_extern::{FnExternConstruct, MethodResolver};
use crate::func::*;
#[cfg(feature = "sys")]
use crate::sys::*;

use virtual_exec_extern::resolve;

pub static BASIC: LazyLock<MethodResolver> = LazyLock::new(||
    resolve!(
        ("push_array", PushArray),
        ("pop_array", PopArray),
        ("arr_get_from_idx", ArrGetFromIdx),
        ("create_array", CreateArray),
        ("arr_get_len", ArrGetLen),
        ("concat", Concat)
    )
);

#[cfg(feature = "sys")]
pub static SYS: LazyLock<MethodResolver> = LazyLock::new(||
    resolve!(
        ("print", Print),
        ("println", PrintLn),
        ("arr_get_from_idx", ArrGetFromIdx),
        ("create_array", CreateArray),
        ("arr_get_len", ArrGetLen),
        ("concat", Concat)
    )
);