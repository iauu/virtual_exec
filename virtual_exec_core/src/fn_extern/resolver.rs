use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};
use crate::fn_extern::{FnExtern};

#[derive(Clone)]
pub struct MethodResolver {
    funcs: HashMap<String, Arc<RwLock<dyn FnExtern + Send + Sync>>>
}

impl<'a> MethodResolver {
    pub fn get(&self, name: &str) -> Option<Arc<RwLock<dyn FnExtern + Send + Sync>>> {
        self.funcs.get(name).map(|r| Arc::clone(r))
    }

    pub fn new(funcs: HashMap<String, Arc<RwLock<dyn FnExtern + Send + Sync>>>) -> MethodResolver {
        Self {
            funcs
        }
    }
}

impl Debug for MethodResolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let method_names: Vec<&String> = self.funcs.keys().collect();

        f.debug_struct("MethodResolver")
            .field("methods", &method_names)
            .finish()
    }
}