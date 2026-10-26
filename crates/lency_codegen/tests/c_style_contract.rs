use inkwell::{context::Context, OptimizationLevel};

extern "C" fn unexpected_panic(_: *const u8, _: u32) -> ! {
    std::process::abort()
}

fn run(source: &str) -> i64 {
    let mut program = lency_syntax::parser::parse(source).expect("valid syntax");
    lency_sema::analyze(&mut program).expect("valid semantics");
    let context = Context::create();
    let module = lency_codegen::compile_to_module(&program, &context, "contract", Some(source))
        .expect("valid LLVM");
    let engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .expect("JIT");
    if let Some(panic) = module.get_function("lency_panic_at") {
        engine.add_global_mapping(&panic, unexpected_panic as *const () as usize);
    }
    unsafe {
        let main = engine
            .get_function::<unsafe extern "C" fn() -> i64>("__lency_main")
            .expect("main");
        main.call()
    }
}

#[test]
fn nested_shadowing_restores_bindings_but_preserves_outer_assignments() {
    assert_eq!(run("int main() { var x = 1 var total = 0 { var x = 9 total = total + x { var x = 4 total = total + x } total = total + x } return x + total }"), 23);
}

#[test]
fn loop_initializer_does_not_replace_outer_binding() {
    assert_eq!(run("int main() { var i = 9 var total = 0 for (int i = 0 i < 3 i = i + 1) { total = total + i } return i + total }"), 12);
}

#[test]
fn fully_terminated_nested_branches_have_no_live_merge() {
    assert_eq!(run("int pick(int x) { if x > 0 { { return 7 } } else { if x == 0 { return 8 } else { return 9 } } } int main() { return pick(1) + pick(0) + pick(-1) }"), 24);
}

#[test]
fn c_arrays_execute_and_keep_outer_dimension_first() {
    assert_eq!(
        run("int main() { int data[2][3] = {{1, 2, 3}, {4, 5, 6}} return data[1][2] }"),
        6
    );
}

#[test]
fn panic_is_a_non_returning_statement_and_keeps_its_source_line() {
    let source = "int stop() {\n panic(\"stopped\")\n}\nint main() { return stop() }";
    let mut program = lency_syntax::parser::parse(source).unwrap();
    lency_sema::analyze(&mut program).unwrap();
    let ir = lency_codegen::compile_to_ir(&program, "panic_contract", Some(source)).unwrap();
    assert!(ir.contains("i32 2)"));
    assert!(ir.contains("unreachable"));
    assert!(ir.contains("@lency_panic_at"));
}

#[test]
fn failed_numeric_conversions_check_ffi_status_before_continuing() {
    for (call, message) in [
        ("parse_int(\"invalid\")", "Invalid integer conversion"),
        ("parse_float(\"invalid\")", "Invalid float conversion"),
    ] {
        let source = format!("int main() {{ var value = {call} return 0 }}");
        let mut program = lency_syntax::parser::parse(&source).unwrap();
        lency_sema::analyze(&mut program).unwrap();
        let ir = lency_codegen::compile_to_ir(&program, "parse_contract", Some(&source)).unwrap();
        assert!(ir.contains("load i32"));
        assert!(ir.contains("parse_failed"));
        assert!(ir.contains(message));
    }
}
