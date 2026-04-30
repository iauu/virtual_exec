pub mod func;

#[cfg(feature = "sys")]
pub mod sys;

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use virtual_exec_core::fn_extern::{FnExtern, FnExternConstruct, MethodResolver};
use crate::func::*;
#[cfg(feature = "sys")]
use crate::sys::*;

macro_rules! add_item {
    ($map:expr, $name:expr, $item:ident) => {
        $map.insert($name.to_string(), ::std::sync::Arc::new($item::new()));
    };
}

pub static BASIC: LazyLock<MethodResolver> = LazyLock::new(||{ 
    let mut map: HashMap<String, Arc<dyn FnExtern + Send + Sync>> = HashMap::new();
    add_item!(map, "push_array", PushArray);
    add_item!(map, "pop_array", PopArray);
    add_item!(map, "arr_get_from_idx", ArrGetFromIdx);
    add_item!(map, "create_array", CreateArray);
    add_item!(map, "arr_get_len", ArrGetLen);
    MethodResolver::new(
        map
    )
});

#[cfg(feature = "sys")]
pub static SYS: LazyLock<MethodResolver> = LazyLock::new(||{
    let mut map: HashMap<String, Arc<dyn FnExtern + Send + Sync>> = HashMap::new();
    add_item!(map, "print", Print);
    MethodResolver::new(
        map
    )
});