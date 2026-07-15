#![cfg(feature = "parse")]

use virtual_exec_parser::parser::parse;
use virtual_exec_core::sequential::compile::compile;
use virtual_exec_core::sequential::instructions::Instruction;

#[test]
fn test_value_creation_and_downcast() {
    let code = "a = 1; b = 2; c = 3; if a != b {d = 2;} d += d; d;";
    let parsed = parse(code).unwrap();
    let compiled = compile(&parsed);
    assert_eq!(compiled, vec![
        Instruction::LoadName(Box::from("a")),
        Instruction::LoadLitInt(1),
        Instruction::Assign,
        Instruction::LoadName(Box::from("b")),
        Instruction::LoadLitInt(2),
        Instruction::Assign,
        Instruction::LoadName(Box::from("c")),
        Instruction::LoadLitInt(3),
        Instruction::Assign,
        Instruction::LoadName(Box::from("a")),
        Instruction::LoadName(Box::from("b")),
        Instruction::NotEq,
        Instruction::JmpZ(16),
        Instruction::LoadName(Box::from("d")),
        Instruction::LoadLitInt(2),
        Instruction::Assign,
        Instruction::LoadName(Box::from("d")),
        Instruction::LoadName(Box::from("d")),
        Instruction::LoadName(Box::from("d")),
        Instruction::Add,
        Instruction::Assign,
        Instruction::LoadName(Box::from("d")),
        Instruction::Pop
    ]);
}
