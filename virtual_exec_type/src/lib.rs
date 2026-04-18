#[macro_use]
pub mod op;

pub mod ast;
pub mod base;
pub mod builtin;
pub mod error;
pub mod exec_ctx;
pub mod export;
mod op_impl;

use std::sync::{LazyLock, Arc, Mutex};

static static_bumpalo: LazyLock<Arc<Mutex<bumpalo::Bump>>> = LazyLock::new(|| Arc::new(Mutex::new(bumpalo::Bump::new())));