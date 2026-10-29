use inkwell::{context::Context, OptimizationLevel};
mod support;

extern "C" fn unexpected_panic(_: *const u8, _: u32) -> ! {
    std::process::abort()
}

fn run(source: &str) -> i64 {
    let mut program = lency_syntax::parser::parse(source).expect("syntax");
    lency_sema::analyze(&mut program).expect("semantics");
    let context = Context::create();
    let module = lency_codegen::compile_to_module(&program, &context, "core_values", Some(source))
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
fn mixed_numeric_results_keep_the_promoted_type() {
    assert_eq!(run("float add() { return 1 + 2.5 } int main() { var value = (1 + 2.5) * 2.0 if value == 7.0 && add() == 3.5 { return 1 } return 0 }"), 1);
}

#[test]
fn array_writes_reach_original_storage() {
    assert_eq!(
        run("int main() { int values[2] = {1, 2} values[0] = 7 return values[0] }"),
        7
    );
    assert_eq!(run("struct Grid { int cells[2][2] } int main() { var grid = Grid { cells: [[1, 2], [3, 4]] } grid.cells[1][0] = 7 return grid.cells[1][0] }"), 7);
}

#[test]
fn array_index_side_effects_run_once_before_the_rhs() {
    assert_eq!(run("struct Counter { int count } int next(Counter counter) { counter.count = counter.count + 1 return 0 } int main() { var counter = Counter { count: 0 } int values[1] = {0} values[next(counter)] = counter.count return values[0] + counter.count }"), 2);
}

#[test]
fn remainder_keeps_numeric_types_and_avoids_signed_division_trap() {
    assert_eq!(
        run("int main() { var minimum = -9223372036854775807 - 1 return minimum % -1 }"),
        0
    );
    assert_eq!(run("int main() { if 7 % 2.5 == 2.0 && 7.5 % 2 == 1.5 && 7.5 % 2.5 == 0.0 { return 1 } return 0 }"), 1);
}

#[test]
fn checked_nullable_arrays_write_their_payload_storage() {
    assert_eq!(run("int main() { int[2]? values = [1, 2] if values != null { values[0] = 7 return values[0] } return 0 }"), 7);
}

#[test]
fn match_arms_use_actual_end_blocks_and_compatible_values() {
    assert_eq!(
        run(
            "int main() { int? value = null return match 0 { case 0 => value ?? 7, case _ => 9 } }"
        ),
        7
    );
    assert_eq!(run("int main() { float value = match 1 { case 0 => 2.5, case _ => 7 } if value == 7.0 { return 1 } return 0 }"), 1);
    assert_eq!(
        run("int main() { int? value = null return match value { case null => 7, case _ => 9 } }"),
        7
    );
}

#[test]
fn enum_bindings_and_nested_payload_patterns_keep_handles_and_boolean_width() {
    assert_eq!(run("enum Item { Value(int) } int main() { var item = Item.Value(7) var same = match item { case other => other } return match same { case Item.Value(value) => value } }"), 7);
    assert_eq!(run("enum Flag { Bit(bool) } enum Item { Wrap(Flag) } int main() { var item = Item.Wrap(Flag.Bit(true)) return match item { case Item.Wrap(Flag.Bit(true)) => 7, case Item.Wrap(Flag.Bit(false)) => 9 } }"), 7);
    assert_eq!(
        run("int main() { return match 1.5 { case 1.5 => 7, case _ => 9 } }"),
        7
    );
}
