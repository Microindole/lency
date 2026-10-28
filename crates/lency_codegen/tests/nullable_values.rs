use inkwell::{context::Context, OptimizationLevel};
mod support;

extern "C" fn unexpected_panic(_: *const u8, _: u32) -> ! {
    std::process::abort()
}

fn run(source: &str) -> i64 {
    let mut program = lency_syntax::parser::parse(source).expect("syntax");
    lency_sema::analyze(&mut program).expect("semantics");
    let context = Context::create();
    let module = lency_codegen::compile_to_module(&program, &context, "nullable", Some(source))
        .expect("LLVM");
    let engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    support::register_trace(&engine, &module);
    if let Some(function) = module.get_function("lency_panic_at") {
        engine.add_global_mapping(&function, unexpected_panic as *const () as usize);
    }
    unsafe {
        let main = engine
            .get_function::<unsafe extern "C" fn() -> i64>("__lency_main")
            .unwrap();
        main.call()
    }
}

#[test]
fn scalar_values_preserve_zero_false_and_null() {
    for (body, expected) in [
        ("int? value = null return value ?? 9", 9),
        ("int? value = 0 return value ?? 9", 0),
        ("int? value = 7 value = null return value ?? 9", 9),
        ("int? value = null value = 7 return value ?? 9", 7),
        (
            "bool? value = false if value ?? true { return 1 } return 0",
            0,
        ),
        (
            "float? value = 0.0 if (value ?? 9.0) == 0.0 { return 1 } return 0",
            1,
        ),
    ] {
        assert_eq!(run(&format!("int main() {{ {body} }}")), expected);
    }
}

#[test]
fn nullable_parameters_and_returns_keep_owned_payloads() {
    assert_eq!(run("int? keep(int? value) { return value } int? make() { return 7 } int main() { var first = keep(0) var second = keep(null) var third = make() return (first ?? 10) + (second ?? 2) + (third ?? 10) }"), 9);
    assert_eq!(run("float? keep(float? value) { return value } int main() { var value = keep(2) if (value ?? 9.0) == 2.0 { return 1 } return 0 }"), 1);
}

#[test]
fn nullable_copies_and_comparisons_keep_value_semantics() {
    assert_eq!(run("int main() { int? first = 7 int? second = first first = 2 return (first ?? 9) + (second ?? 9) }"), 9);
    assert_eq!(run("int main() { float? first = 2.0 float? second = 2.0 if first == second { return 1 } return 0 }"), 1);
    assert_eq!(run("int main() { string? first = \"text\" string? second = \"text\" if first == second { return 1 } return 0 }"), 1);
}

#[test]
fn checked_scalar_values_work_in_operators_and_nonnullable_calls() {
    assert_eq!(run("int twice(int value) { return value * 2 } int pick(int? value) { if value != null { return twice(value) + -value } return 9 } int main() { return pick(7) }"), 7);
}

#[test]
fn optional_scalar_equality_compares_payloads_not_allocations() {
    for (left, right, equal) in [
        ("7", "7", 1),
        ("7", "8", 0),
        ("null", "null", 1),
        ("null", "7", 0),
        ("7", "null", 0),
    ] {
        assert_eq!(run(&format!("int main() {{ int? left = {left} int? right = {right} if left == right {{ return 1 }} return 0 }}")), equal);
        assert_eq!(run(&format!("int main() {{ int? left = {left} int? right = {right} if left != right {{ return 1 }} return 0 }}")), 1 - equal);
    }
}

#[test]
fn safe_navigation_distinguishes_absence_from_scalar_zero() {
    assert_eq!(run("struct Item { int value bool flag float ratio int? nested } int main() { Item? missing = null var absent = missing?.value var item = Item { value: 0, flag: false, ratio: 0.0, nested: 7 } var value = item?.value var nested = item?.nested if absent != null { return 1 } if value == null { return 2 } if item?.flag ?? true { return 3 } if (item?.ratio ?? 9.0) != 0.0 { return 4 } item.nested = null if item?.nested != null { return 5 } return (absent ?? 2) + (value ?? 10) + (nested ?? 10) }"), 9);
}

#[test]
fn coalescing_stays_lazy_and_preserves_nullable_or_promoted_results() {
    assert_eq!(run("int stop() { panic(\"must not run\") } int main() { int? value = 7 return value ?? stop() }"), 7);
    assert_eq!(
        run("int main() { int? left = null int? right = null return (left ?? right) ?? 9 }"),
        9
    );
    assert_eq!(
        run("int main() { int? left = 7 int? right = null return (left ?? right) ?? 9 }"),
        7
    );
    assert_eq!(
        run("int main() { int? value = 2 if (value ?? 3.5) == 2.0 { return 1 } return 0 }"),
        1
    );
    assert_eq!(run("int main() { return null ?? 9 }"), 9);
}

#[test]
fn methods_arrays_and_checked_boolean_conditions_use_consistent_payloads() {
    assert_eq!(run("struct Holder { int value } impl Holder { int? keep(int? value) { return value } } int main() { var holder = Holder { value: 1 } var result = holder.keep(7) return result ?? 9 }"), 7);
    assert_eq!(run("int main() { int? first = 0 int? second = null int? values[2] = {first, second} return (values[0] ?? 9) + (values[1] ?? 2) }"), 2);
    assert_eq!(run("int main() { bool? ready = false if ready != null { if ready { return 9 } if !ready && true { return 1 } } return 0 }"), 1);
    assert_eq!(
        run("int main() { int values[2] = {1, 2} return values?.length ?? 9 }"),
        2
    );
}

#[test]
fn narrowed_reads_keep_derived_array_element_types_consistent() {
    assert_eq!(run("int main() { int? value = 7 if value != null { int values[2] = {value, value} return values[0] + values[1] } return 0 }"), 14);
}
