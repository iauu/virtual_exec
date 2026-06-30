#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod sequential;
pub mod machine;
pub mod fn_extern;
#[cfg(feature = "parse")]
pub use virtual_exec_parser::parser::parse;
pub use crate::sequential::compile::compile;
pub use crate::machine::Machine;


pub(crate) use virtual_exec_type::HashMap;