use crate::HashMap;
use crate::fn_extern::FnExtern;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct MethodResolver {
    funcs: HashMap<String, Arc<dyn FnExtern + Send + Sync>>,
}

impl<'a> MethodResolver {
    pub fn get(&self, name: &str) -> Option<Arc<dyn FnExtern + Send + Sync>> {
        self.funcs.get(name).map(|r| Arc::clone(r))
    }

    pub fn new(funcs: HashMap<String, Arc<dyn FnExtern + Send + Sync>>) -> MethodResolver {
        Self { funcs }
    }

    pub fn get_pair(&self) -> Vec<(String, usize)> {
        self.funcs
            .iter()
            .map(|(a, b)| (a.clone(), b.get_size()))
            .collect()
    }
}

impl Debug for MethodResolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let method_names: Vec<&String> = self.funcs.keys().collect();

        f.debug_struct("MethodResolver")
            .field("methods", &method_names)
            .finish()
    }
}
