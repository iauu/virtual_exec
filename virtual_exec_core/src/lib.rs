pub mod sequential;
pub mod machine;
mod fn_extern;

pub use virtual_exec_parser::parser::parse;
pub use crate::sequential::compile::compile;
pub use crate::machine::Machine;