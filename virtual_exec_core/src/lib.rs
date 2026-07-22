#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod fn_extern;
pub mod machine;
pub mod sequential;
pub use crate::machine::Machine;
pub use crate::sequential::compile::compile;
#[cfg(feature = "parse")]
pub use virtual_exec_parser::parser::parse;

pub(crate) use virtual_exec_type::HashMap;
