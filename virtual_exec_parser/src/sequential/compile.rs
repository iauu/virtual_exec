use virtual_exec_type::ast::core::{Module, Expr, ASTNode, Literal, Stmt, AssignExpr};
use crate::sequential::instructions::{Instruction, InstructionBuilder};

trait GetInstruction : ASTNode {
    fn inst(&self) -> Vec<InstructionBuilder>;
}

