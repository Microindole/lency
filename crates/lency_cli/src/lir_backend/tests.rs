use super::*;

#[test]
fn test_compile_min_lir_to_llvm() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  var %x = 1
  var %y = 2
  %t0 = add %x, %y
  store %x, %t0
  %t1 = cmp_gt %x, 0
  br %t1, then_0, else_1
then_0:
  ret %x
else_1:
  ret 0
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("define i32 @main(i32 %process_argc, i8** %process_argv)"));
    assert!(ir.contains("alloca i64"));
    assert!(ir.contains("icmp sgt i64"));
    assert!(ir.contains("ret i32"));
}

#[test]
fn test_compile_lir_call_external_function() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  var %x = 5
  %t0 = call %foo(%x, 1)
  ret %t0
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("declare i64 @foo(i64, i64)"));
    assert!(ir.contains("call i64 @foo("));
}

#[test]
fn test_compile_lir_maps_arg_count_runtime_symbol() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  %t0 = call %arg_count()
  ret %t0
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("define i64 @lency_process_arg_count()"));
    assert!(ir.contains("call i64 @lency_process_arg_count()"));
}

#[test]
fn test_compile_lir_maps_arg_at_runtime_symbol() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  %t0 = call %arg_at(0)
  ret %t0
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("define i8* @lency_process_arg_at(i64 %index)"));
    assert!(ir.contains("call i8* @lency_process_arg_at(i64 0)"));
}

#[test]
fn test_compile_lir_maps_int_to_string_runtime_symbol() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  %t0 = call %int_to_string(7)
  ret %t0
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("declare i8* @lency_int_to_string(i64)"));
    assert!(ir.contains("call i8* @lency_int_to_string(i64 7)"));
}

#[test]
fn test_compile_lir_get_to_string_lowering() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  var %x = 7
  %t0 = get %x.to_string
  %t1 = call %t0()
  ret %t1
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("declare i8* @lency_int_to_string(i64)"));
    assert!(ir.contains("call i8* @lency_int_to_string(i64"));
}

#[test]
fn test_compile_lir_call_through_member_temp_no_args() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  var %x = 7
  %t0 = get %x.to_string
  %t1 = call %t0()
  ret %t1
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("declare i8* @lency_int_to_string(i64)"));
    assert!(ir.contains("call i8* @lency_int_to_string(i64"));
    assert!(!ir.contains("call i64 @t0("));
}

#[test]
fn test_compile_lir_get_len_lowering() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  %t0 = call %int_to_string(7)
  %t1 = get %t0.len
  %t2 = call %t1()
  ret %t2
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("declare i64 @lency_string_len(i8*)"));
    assert!(ir.contains("call i64 @lency_string_len(i8*"));
}

#[test]
fn test_compile_lir_call_member_substr_lowering() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  %t0 = call %int_to_string(12345)
  %t1 = get %t0.substr
  %t2 = call %t1(1, 2)
  ret %t2
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("declare i8* @lency_string_substr(i8*, i64, i64)"));
    assert!(ir.contains("call i8* @lency_string_substr(i8*"));
}

#[test]
fn test_compile_lir_call_member_split_lowering() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  %t0 = call %int_to_string(12345)
  %t1 = call %int_to_string(3)
  %t2 = get %t0.split
  %t3 = call %t2(%t1)
  ret %t3
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("declare i8* @lency_string_split(i8*, i8*)"));
    assert!(ir.contains("call i8* @lency_string_split(i8*"));
}

#[test]
fn test_compile_lir_call_member_join_lowering() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  %t0 = call %arg_at(0)
  %t1 = call %arg_at(0)
  %t2 = get %t0.join
  %t3 = call %t2(%t1)
  ret %t3
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("declare i8* @lency_string_join(i8*, i8*)"));
    assert!(ir.contains("call i8* @lency_string_join(i8*"));
}

#[test]
fn test_compile_lir_call_member_generic_fallback() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  %t0 = call %int_to_string(12345)
  %t1 = call %int_to_string(5)
  %t2 = get %t0.contains
  %t3 = call %t2(%t1)
  ret %t3
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("declare i1 @contains(i8*, i8*)"));
    assert!(ir.contains("call i1 @contains(i8*"));
}

#[test]
fn test_compile_lir_string_match_compare() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  var %x = 0
  %t0 = cmp_str_eq %x, "42"
  ret %t0
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("@.str.0 = private unnamed_addr constant [3 x i8] c\"42\\00\""));
    assert!(ir.contains("declare i64 @lency_string_eq(i8*, i8*)"));
    assert!(ir.contains("call i64 @lency_string_eq(i8*"));
}

#[test]
fn test_compile_lir_string_not_equal_compare() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  var %x = 0
  %t0 = cmp_str_ne %x, "42"
  ret %t0
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("declare i64 @lency_string_eq(i8*, i8*)"));
    assert!(ir.contains("call i64 @lency_string_eq(i8*"));
    assert!(ir.contains("icmp eq i64"));
}

#[test]
fn test_compile_lir_enum_runtime_calls() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  %t0 = call %lency_enum_new0(2)
  %t1 = call %lency_enum_push(%t0, 9)
  %t2 = call %lency_enum_tag(%t0)
  %t3 = call %lency_enum_payload(%t0, 0)
  %t4 = add %t2, %t3
  ret %t4
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("declare i64 @lency_enum_new0(i64)"));
    assert!(ir.contains("declare i64 @lency_enum_push(i64, i64)"));
    assert!(ir.contains("declare i64 @lency_enum_tag(i64)"));
    assert!(ir.contains("declare i64 @lency_enum_payload(i64, i64)"));
    assert!(ir.contains("call i64 @lency_enum_new0(i64 2)"));
    assert!(ir.contains("call i64 @lency_enum_push(i64 %t0, i64 9)"));
}

#[test]
fn test_compile_lir_enum_runtime_calls_multi_push() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  %t0 = call %lency_enum_new0(5)
  %t1 = call %lency_enum_push(%t0, 1)
  %t2 = call %lency_enum_push(%t0, 2)
  %t3 = call %lency_enum_push(%t0, 3)
  %t4 = call %lency_enum_push(%t0, 4)
  %t5 = call %lency_enum_payload(%t0, 3)
  ret %t5
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("declare i64 @lency_enum_new0(i64)"));
    assert!(ir.contains("declare i64 @lency_enum_push(i64, i64)"));
    assert!(ir.contains("call i64 @lency_enum_push(i64 %t0, i64 4)"));
    assert!(ir.contains("call i64 @lency_enum_payload(i64"));
}

#[test]
fn test_compile_lir_void_vec_runtime_calls() {
    let src = r#"
; lencyc-lir v0
func main {
entry:
  %t0 = call %lency_vec_new(2)
  call %lency_vec_push(%t0, 1)
  call %lency_vec_push(%t0, 2)
  %t1 = call %lency_vec_get(%t0, 1)
  call %lency_vec_set(%t0, 1, 3)
  ret %t1
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("declare i8* @lency_vec_new(i64)"));
    assert!(ir.contains("declare void @lency_vec_push(i8*, i64)"));
    assert!(ir.contains("declare i64 @lency_vec_get(i8*, i64)"));
    assert!(ir.contains("declare void @lency_vec_set(i8*, i64, i64)"));
    assert!(ir.contains("call void @lency_vec_push(i8*"));
    assert!(ir.contains("call void @lency_vec_set(i8*"));
}

#[test]
fn test_compile_lir_multi_function_ptr_signature() {
    let src = r#"
; lencyc-lir v0
func main() -> i64 {
entry:
  %t0 = call %make_pair(1, 2)
  %t1 = call %bump_right(%t0)
  ret %t1
}

func make_pair(%left: i64, %right: i64) -> ptr {
entry:
  %t0 = call %lency_vec_new(2)
  call %lency_vec_push(%t0, %left)
  call %lency_vec_push(%t0, %right)
  ret %t0
}
func bump_right(%p: ptr) -> i64 {
entry:
  %t0 = call %lency_vec_get(%p, 1)
  %t1 = add %t0, 1
  call %lency_vec_set(%p, 1, %t1)
  %t2 = call %lency_vec_get(%p, 0)
  %t3 = add %t2, %t1
  ret %t3
}
"#;
    let result = compile_lir_to_llvm_ir(src);
    assert!(result.is_ok(), "lir compile failed: {:?}", result.err());
    let ir = result.unwrap_or_default();
    assert!(ir.contains("define i32 @main(i32 %process_argc, i8** %process_argv)"));
    assert!(ir.contains("define i8* @make_pair(i64 %left, i64 %right)"));
    assert!(ir.contains("define i64 @bump_right(i8* %p)"));
    assert!(ir.contains("call i8* @make_pair(i64 1, i64 2)"));
    assert!(ir.contains("call i64 @bump_right(i8*"));
}

#[test]
fn test_compile_lir_uses_unique_string_globals_across_functions() {
    let src = r#"
; lencyc-lir v0
func first() -> ptr {
entry:
  ret "first"
}

func second() -> ptr {
entry:
  ret "second"
}
"#;
    let ir = compile_lir_to_llvm_ir(src).expect("multi-function string LIR should compile");
    assert_eq!(ir.matches("@.str.0 =").count(), 1);
    assert_eq!(ir.matches("@.str.1 =").count(), 1);
}

#[test]
fn test_compile_lir_keeps_commas_inside_string_operands() {
    let src = r#"
; lencyc-lir v0
func message() -> ptr {
entry:
  %t0 = add "left, middle", ", right"
  ret %t0
}
"#;
    let ir = compile_lir_to_llvm_ir(src).expect("string commas must not split LIR operands");
    assert!(ir.contains("define i8* @message()"));
}
