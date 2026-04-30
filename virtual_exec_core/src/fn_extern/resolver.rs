use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::fn_extern::{FnExtern};

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