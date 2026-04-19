use virtual_exec_type::base::{ValueContainer, ValueKind};
use virtual_exec_type::builtin::VirPyInt;
use virtual_exec_parser::parser::parse;
use virtual_exec_parser::sequential::compile::compile;
use virtual_exec_parser::sequential::instructions::Instruction;

#[test]
fn test_value_creation_and_downcast() {
    let code = "a = 1; b = 2; c = 3; if a != b {d = 2;} d += d; d;";
    let parsed = parse(code).unwrap();
    let compiled = compile(&parsed);
    assert_eq!(compiled, vec![
        Instruction::LoadName("a".to_string()),
        Instruction::LoadLitInt(1),
        Instruction::Assign,
        Instruction::LoadName("b".to_string()),
        Instruction::LoadLitInt(2),
        Instruction::Assign,
        Instruction::LoadName("c".to_string()),
        Instruction::LoadLitInt(3),
        Instruction::Assign,
        Instruction::LoadName("a".to_string()),
        Instruction::LoadName("b".to_string()),
        Instruction::NotEq,
        Instruction::JmpZ(16),
        Instruction::LoadName("d".to_string()),
        Instruction::LoadLitInt(2),
        Instruction::Assign,
        Instruction::LoadName("d".to_string()),
        Instruction::LoadName("d".to_string()),
        Instruction::LoadName("d".to_string()),
        Instruction::Add,
        Instruction::Assign,
        Instruction::LoadName("d".to_string()),
        Instruction::Pop
    ]);
}
