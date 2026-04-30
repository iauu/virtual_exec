pub mod func;

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};
use virtual_exec_core::fn_extern::{FnExtern, FnExternConstruct, MethodResolver};
use crate::func::*;

macro_rules! add_item {
    ($map:expr, $name:expr, $item:ident) => {
        $map.insert($name.to_string(), ::std::sync::Arc::new(::std::sync::RwLock::new($item::new())));
    };
}

static BASIC: LazyLock<MethodResolver> = LazyLock::new(||{ 
    let mut map: HashMap<String, Arc<RwLock<dyn FnExtern + Send + Sync>>> = HashMap::new();
    add_item!(map, "push_array", PushArray);
    add_item!(map, "pop_array", PopArray);
    MethodResolver::new(
        map
    )
});