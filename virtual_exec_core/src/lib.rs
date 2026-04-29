pub mod sequential;
pub mod machine;
pub use virtual_exec_parser::parser::parse;
pub use crate::sequential::compile::compile;
pub use crate::machine::Machine;