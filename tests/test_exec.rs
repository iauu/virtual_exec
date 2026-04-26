// use virtual_exec::interpreted_exec;
// use virtual_exec_type::exec_ctx::RsValue;
// 
// #[test]
// fn test_simple_assignment() {
//     let code = "a = 1; b = 2; c = 3; if a != b {d = 2;} d += d; d;";
//     let result = interpreted_exec(code, 100).unwrap();
//     assert_eq!(result.get("a"), Some(&RsValue::Int(1)));
//     assert_eq!(result.get("d"), Some(&RsValue::Int(4)));
// }
