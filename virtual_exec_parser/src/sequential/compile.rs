use virtual_exec_type::ast::core::{Module, Expr, Literal, Stmt, AssignExpr, Node, BinaryOperator, UnaryOperator};
use crate::sequential::instructions::Instruction;
use crate::sequential::instructions::Instruction::{Jmp, JmpNz, JmpZ, LoadLitBool};

pub fn compile(module: &Module) -> Vec<Instruction> {
    module.inst(0)
}

pub trait GetInstruction {
    fn inst(&self, offset: u64) -> Vec<Instruction>;
}

impl GetInstruction for Module {
    fn inst(&self, offset: u64) -> Vec<Instruction> {
        let mut inst = Vec::new();
        let mut offset = offset;
        for stmt in self.body.clone() {
            let curr = stmt.kind.inst(offset);
            offset += curr.len() as u64;
            inst.extend(curr);
        }
        inst
    }
}

impl GetInstruction for &Vec<Stmt> {
    fn inst(&self, offset: u64) -> Vec<Instruction> {
        let mut inst = Vec::new();
        let mut offset = offset;
        for stmt in *self {
            let curr = stmt.inst(offset);
            offset += curr.len() as u64;
            inst.extend(curr);
        }
        inst
    }
}

impl GetInstruction for &Vec<Node<Stmt>> {
    fn inst(&self, offset: u64) -> Vec<Instruction> {
        let mut inst = Vec::new();
        let mut offset = offset;
        for stmt in *self {
            let curr = stmt.kind.inst(offset);
            offset += curr.len() as u64;
            inst.extend(curr);
        }
        inst
    }
}

impl GetInstruction for Stmt {
    fn inst(&self, offset: u64) -> Vec<Instruction> {
        match self {
            Stmt::Expression(expr) => { 
                let mut base = expr.kind.inst(offset);
                base.push(Instruction::Pop); // The expr have to be discarded
                base
            },
            Stmt::Assign { target, value } => {
                let target_inst = target.kind.inst(offset);
                let value_inst = value.kind.inst(offset + target_inst.len() as u64);
                let mut inst = Vec::new();
                inst.extend(target_inst);
                inst.extend(value_inst);
                inst.push(Instruction::Assign);
                inst
            },
            Stmt::Scoped(stmts) => {
                let mut inst = Vec::new();
                let mut offset = offset;
                for stmt in stmts.clone() {
                    let curr = stmt.kind.inst(offset);
                    offset += curr.len() as u64;
                    inst.extend(curr);
                }
                inst
            },
            Stmt::If {
                test,
                body,
                otherwise,
            } => {
                let mut offset = offset;
                let test_inst = test.kind.inst(offset);
                offset += test_inst.len() as u64;
                // JmpZ (+1 => 1 hidden)
                let body_inst = body.inst(offset + 1);
                offset += body_inst.len() as u64;
                match otherwise {
                    None => {
                        let mut inst: Vec<Instruction> = Vec::new();
                        inst.extend(test_inst);
                        inst.push(JmpZ(offset+1));
                        inst.extend(body_inst);
                        inst
                    },
                    Some(otherwise) => {
                        // Jmp (+1 => 2 hidden)
                        let jzz_target = offset + 2;
                        let otherwise_inst = otherwise.inst(offset + 2);
                        offset += otherwise_inst.len() as u64;
                        let mut inst: Vec<Instruction> = Vec::new();
                        inst.extend(test_inst);
                        inst.push(JmpZ(jzz_target));
                        inst.extend(body_inst);
                        inst.push(Jmp(offset+2));
                        inst.extend(otherwise_inst);
                        inst
                    }
                }
            },
            Stmt::Loop { test, body} => {
                let mut offset = offset;
                let initial = offset;
                let test_inst = test.kind.inst(offset);
                offset += test_inst.len() as u64;
                // JmpZ (non-hidden)
                offset += 1;
                let body_inst = body.inst(offset);
                offset += body_inst.len() as u64;
                // Jmp (non-hidden)
                offset += 1;
                let mut inst: Vec<Instruction> = Vec::new();
                inst.extend(test_inst);
                inst.push(JmpZ(offset));
                inst.extend(body_inst);
                inst.push(Jmp(initial));
                inst
            },
            Stmt::FunctionDef { name, args, body } => {
                let mut offset = offset;
                let mut inst: Vec<Instruction> = Vec::new();
                inst.push(Instruction::LoadName(name.clone().into_boxed_str()));
                offset += 1;
                inst.push(Instruction::LoadDPtr(offset + 3));
                inst.push(Instruction::Assign);
                offset += 2;
                // Jmp (non-hidden)
                offset += 1;
                let mut fn_inst: Vec<Instruction> = Vec::new();
                for arg in args {
                    fn_inst.push(Instruction::LoadName(arg.clone().into_boxed_str()));
                    fn_inst.push(Instruction::Swap);
                    fn_inst.push(Instruction::Assign);
                    offset += 3;
                }
                // [value1] [value0] |inst boundary| [var0] [var1]
                let body_inst = body.inst(offset);
                offset += body_inst.len() as u64;
                fn_inst.extend(body_inst);
                fn_inst.push(Instruction::LoadNone);
                fn_inst.push(Instruction::Ret);
                offset += 2;
                inst.push(Instruction::Jmp(offset));
                inst.extend(fn_inst);
                inst
            },
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    let mut inst = Vec::new();
                    let expr_inst = expr.kind.inst(offset);
                    inst.extend(expr_inst);
                    inst.push(Instruction::Ret);
                    inst
                } else {
                    vec![Instruction::LoadNone, Instruction::Ret]
                }
            }
        }
    }
}

impl GetInstruction for Expr {
    fn inst(&self, offset: u64) -> Vec<Instruction> {
        match self {
            Expr::Literal(l) => l.inst(offset),
            Expr::Wrapped(expr) => expr.kind.inst(offset),
            Expr::Variable(v) => vec![Instruction::LoadName(v.clone().into_boxed_str())],
            Expr::BinaryOp { left, op, right } => {
                let mut inst = Vec::new();
                match op {
                    op @ (BinaryOperator::And | BinaryOperator::Or) => {
                        let mut offset = offset;
                        let left_inst = left.kind.inst(offset);
                        offset += left_inst.len() as u64;
                        inst.extend(left_inst);
                        if let BinaryOperator::And = op {
                            // Doesn't actually need logical and since it is in the jmp behaviour
                            // JmpZ (+1 => 1 hidden)
                            let right_inst = right.kind.inst(offset + 1);
                            offset += right_inst.len() as u64;
                            inst.push(JmpZ(offset+1));
                            inst.extend(right_inst);
                        } else if let BinaryOperator::Or = op {
                            // Doesn't actually need logical or since it is in the jmp behaviour
                            // JmpNz (+1 => 1 hidden)
                            let right_inst = right.kind.inst(offset + 1);
                            offset += right_inst.len() as u64;
                            inst.push(JmpNz(offset+1));
                            inst.extend(right_inst);
                        } else {
                            unreachable!()
                        }
                    },
                    op @ _ => {

                        let left_inst = left.kind.inst(offset);
                        let right_inst = right.kind.inst(offset + left_inst.len() as u64);
                        inst.extend(left_inst);
                        inst.extend(right_inst);
                        match op {
                            BinaryOperator::And | BinaryOperator::Or  => unreachable!(),
                            BinaryOperator::Add => inst.push(Instruction::Add),
                            BinaryOperator::Subtract => inst.push(Instruction::Sub),
                            BinaryOperator::Multiply => inst.push(Instruction::Mul),
                            BinaryOperator::Divide => inst.push(Instruction::Div),
                            BinaryOperator::BitwiseAnd => inst.push(Instruction::BitwiseAnd),
                            BinaryOperator::BitwiseOr => inst.push(Instruction::BitwiseOr),
                            BinaryOperator::Xor => inst.push(Instruction::BitwiseXor),
                            BinaryOperator::Modulo => inst.push(Instruction::Mod),
                            BinaryOperator::Eq => inst.push(Instruction::Eq),
                            BinaryOperator::NotEq => inst.push(Instruction::NotEq),
                            BinaryOperator::Lt => inst.push(Instruction::Lt),
                            BinaryOperator::Lte => inst.push(Instruction::Lte),
                            BinaryOperator::Gt => inst.push(Instruction::Gt),
                            BinaryOperator::Gte => inst.push(Instruction::Gte),
                            BinaryOperator::LeftShift => inst.push(Instruction::Shl),
                            BinaryOperator::RightShift => inst.push(Instruction::Shr),
                        }
                    }
                }
                inst
            },
            Expr::UnaryOp { op, operand } => {
                let mut inst = Vec::new();
                let operand_inst = operand.kind.inst(offset);
                inst.extend(operand_inst);
                match op {
                    UnaryOperator::Positive => inst.push(Instruction::UnaryPlus),
                    UnaryOperator::Negative => inst.push(Instruction::UnaryMinus),
                    UnaryOperator::Not => inst.push(Instruction::Not),
                }
                inst
            },
            Expr::Call { function, args} => {
                let mut inst = Vec::new();
                let mut offset = offset;
                for arg in args.iter().rev() {
                    let arg_inst = arg.kind.inst(offset);
                    offset += arg_inst.len() as u64;
                    inst.extend(arg_inst);
                }
                let function_inst = function.kind.inst(offset);
                offset += function_inst.len() as u64;
                inst.extend(function_inst);
                inst.push(Instruction::Call);
                // [value1] [value0] ([func] [call]) |inst boundary| [var0] [var1]
                inst
            }
        }
    }
}

impl GetInstruction for Literal {
    fn inst(&self, _offset: u64) -> Vec<Instruction> {
        match self {
            Literal::Bool(v) => vec![LoadLitBool(v.clone())],
            Literal::Int(v) => vec![Instruction::LoadLitInt(v.clone())],
            Literal::Float(v) => vec![Instruction::LoadLitFloat(v.clone())],
            Literal::String(v) => vec![Instruction::LoadLitString(v.clone().into_boxed_str())],
            Literal::None => vec![Instruction::LoadNone],
        }
    }
}

impl GetInstruction for AssignExpr {
    fn inst(&self, offset: u64) -> Vec<Instruction> {
        match self {
            AssignExpr::Variable(v) => vec![Instruction::LoadName(v.clone().into_boxed_str())],
            AssignExpr::Wrapped(expr) => expr.kind.inst(offset),
        }
    }
}