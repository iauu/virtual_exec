#[macro_use]
pub mod op;

pub mod ast;
pub mod base;
pub mod error;
mod op_impl;
pub mod mem;

pub mod vm_type {
    use std::collections::HashMap;
    use std::sync::{Arc};
    use async_lock::RwLock;
    use crate::error::ExecutionError;
    use crate::mem::ValuePtr;

    pub type Integer = i64;
    pub type Float = f64;
    pub type Object<'a> = Arc<RwLock<HashMap<String, ValuePtr<'a>>>>;
    pub type Collection<'a> = Arc<RwLock<Vec<ValuePtr<'a>>>>;
    pub type Str = String;
    pub type Boolean = bool;
    pub type None = ();

    pub type Error = ExecutionError;
    pub type Any<'a> = ValuePtr<'a>;
}