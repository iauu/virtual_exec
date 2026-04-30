pub mod func;

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};
use virtual_exec_core::fn_extern::{FnExtern, FnExternConstruct, MethodResolver};
use crate::func::PushArray;

static BASIC: LazyLock<MethodResolver> = LazyLock::new(||{ 
    let mut map: HashMap<String, Arc<RwLock<dyn FnExtern + Send + Sync>>> = HashMap::new();
    map.entry("push_array".to_string()).or_insert(Arc::new(RwLock::new(PushArray::new())));
    MethodResolver::new(
        map
    )
});