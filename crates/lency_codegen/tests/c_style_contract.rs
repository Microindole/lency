use inkwell::{context::Context, OptimizationLevel};
mod support;

thread_local! {
    static PROBE_CALLS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static PROBE_TRACE: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

extern "C" fn probe(value: bool) -> bool {
    PROBE_CALLS.with(|calls| calls.set(calls.get() + 1));
    PROBE_TRACE.with(|trace| trace.set(trace.get() * 10 + if value { 2 } else { 1 }));
    value
}

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
    support::register_trace(&engine, &module);
    if let Some(panic) = module.get_function("lency_panic_at") {
        engine.add_global_mapping(&panic, unexpected_panic as *const () as usize);
    }
    if let Some(function) = module.get_function("probe") {
        engine.add_global_mapping(&function, probe as *const () as usize);
    }
    unsafe {
        let main = engine
            .get_function::<unsafe extern "C" fn() -> i64>("__lency_main")
            .expect("main");
        main.call()
    }
}

#[test]
fn logical_operators_short_circuit_and_evaluate_operands_once() {
    for (operator, left, right, result, calls) in [
        ("&&", false, false, false, 1),
        ("&&", false, true, false, 1),
        ("&&", true, false, false, 2),
        ("&&", true, true, true, 2),
        ("||", false, false, false, 2),
        ("||", false, true, true, 2),
        ("||", true, false, true, 1),
        ("||", true, true, true, 1),
    ] {
        PROBE_CALLS.with(|calls| calls.set(0));
        PROBE_TRACE.with(|trace| trace.set(0));
        let source = format!(
            "extern bool probe(bool value); int main() {{ if probe({left}) {operator} probe({right}) {{ return 1 }} return 0 }}"
        );
        assert_eq!(run(&source), i64::from(result), "{source}");
        assert_eq!(PROBE_CALLS.with(|calls| calls.get()), calls, "{source}");
        let left_digit = if left { 2 } else { 1 };
        let expected_trace = if calls == 1 {
            left_digit
        } else {
            left_digit * 10 + if right { 2 } else { 1 }
        };
        assert_eq!(PROBE_TRACE.with(|trace| trace.get()), expected_trace);
    }
}

#[test]
fn nested_logical_operators_keep_actual_predecessors_and_left_to_right_order() {
    PROBE_CALLS.with(|calls| calls.set(0));
    PROBE_TRACE.with(|trace| trace.set(0));
    assert_eq!(run("extern bool probe(bool value); int main() { if (probe(false) || probe(true)) && (probe(false) || probe(true)) { return 1 } return 0 }"), 1);
    assert_eq!(PROBE_CALLS.with(|calls| calls.get()), 4);
    assert_eq!(PROBE_TRACE.with(|trace| trace.get()), 1212);
}

#[test]
fn logical_guards_skip_unsafe_rhs_and_preserve_checks_on_live_paths() {
    assert_eq!(run("int main() { int values[1] = {7} var zero = 0 var invalid = 10 if zero != 0 && 10 / zero > 1 { return 1 } if !(zero == 0 || values[invalid] == 0) { return 2 } if !(zero == 0 && (10 / 2 == 5 && values[0] == 7)) { return 3 } var i = 0 while i < 1 && values[i] == 7 { i = i + 1 } return i }"), 1);
}

#[test]
fn skipped_operands_still_require_valid_names_and_boolean_types() {
    for expression in [
        "false && 1",
        "true || 1",
        "false && missing",
        "true || missing",
        "true || values[10] == 0",
    ] {
        let source = format!(
            "int main() {{ int values[1] = {{7}} if {expression} {{ return 1 }} return 0 }}"
        );
        let mut program = lency_syntax::parser::parse(&source).unwrap();
        assert!(lency_sema::analyze(&mut program).is_err(), "{source}");
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
