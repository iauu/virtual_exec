#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate core;

#[macro_use]
pub mod op;

pub mod ast;
pub mod base;
pub mod config;
pub mod error;
pub mod ext;
pub mod mem;
mod op_impl;

pub mod vm_type {
    #[cfg(feature = "std")]
    pub(crate) type HashMap<K, V> = std::collections::HashMap<K, V>;

    #[cfg(not(feature = "std"))]
    pub(crate) type HashMap<K, V> = alloc::collections::BTreeMap<K, V>;

    use crate::error::ExecutionError;
    use crate::mem::ValuePtr;
    use alloc::string::{String, ToString};
    use alloc::sync::Arc;
    use alloc::vec::Vec;
    use async_lock::RwLock;

    pub type Integer = i64;
    pub type Float = f64;
    pub type Object<'a> = Arc<RwLock<crate::HashMap<String, ValuePtr<'a>>>>;
    pub type Collection<'a> = Arc<RwLock<Vec<ValuePtr<'a>>>>;
    pub type Str = String;
    pub type Boolean = bool;
    pub type None = ();

    pub type Error = ExecutionError;
    pub type Any<'a> = ValuePtr<'a>;
}

#[cfg(feature = "std")]
pub type HashMap<K, V> = std::collections::HashMap<K, V>;

#[cfg(not(feature = "std"))]
pub type HashMap<K, V> = alloc::collections::BTreeMap<K, V>;

pub mod __private {
    pub use alloc::string::String;
    pub use alloc::sync::Arc;
}
