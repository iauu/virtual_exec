pub mod func;

#[cfg(feature = "sys")]
pub mod sys;

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use virtual_exec_core::fn_extern::{FnExternConstruct, MethodResolver};
use crate::func::*;
#[cfg(feature = "sys")]
use crate::sys::*;

#[macro_export]
macro_rules! add_item {
    ($map:expr, $name:expr, $item:ident) => {
        {
            use ::virtual_exec_core::fn_extern::FnExternConstruct;
            $map.insert($name.to_string(), ::std::sync::Arc::new($item::new()));
        };
    };
}

#[macro_export]
macro_rules! resolve {
    ($(($name:expr, $item:ident)),*) => {

        {
            let mut map: ::std::collections::HashMap<::std::string::String, Arc<dyn ::virtual_exec_core::fn_extern::FnExtern + ::core::marker::Send + ::core::marker::Sync>> = ::std::collections::HashMap::new();
            $($crate::add_item!(map, $name, $item);)*
            ::virtual_exec_core::fn_extern::MethodResolver::new(
                map
            )
        }
    };
}

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