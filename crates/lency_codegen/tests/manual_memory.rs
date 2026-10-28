use inkwell::{context::Context, OptimizationLevel};
use std::process::Command;
mod support;

fn run(source: &str) -> i64 {
    let mut program = lency_syntax::parser::parse(source).unwrap();
    lency_sema::analyze(&mut program).unwrap();
    let context = Context::create();
    let module =
        lency_codegen::compile_to_module(&program, &context, "manual_memory.lcy", Some(source))
            .unwrap();
    let engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    support::register_trace(&engine, &module);
    if let Some(function) = module.get_function("lency_panic_at") {
        engine.add_global_mapping(
            &function,
            lency_runtime::lency_panic_at as *const () as usize,
        );
    }
    unsafe {
        let main = engine
            .get_function::<unsafe extern "C" fn() -> i64>("__lency_main")
            .unwrap();
        main.call()
    }
}

#[test]
fn nullable_boxes_and_returned_structures_can_be_released() {
    assert_eq!(run("struct Item { int value } Item make() { return Item { value: 7 } } int main() { var item = make() int? value = 0 int result = item.value if value != null { result = result + value free(value) } value = null free(value) free(item) free(null) return result }"), 7);
    assert_eq!(run("enum State { Idle, Value(int) } int main() { var state = State.Value(7) free(state) return 0 }"), 0);
}

#[test]
fn shallow_release_does_not_destroy_shared_children() {
    assert_eq!(run("struct Child { int value } struct Parent { Child child } int main() { var child = Child { value: 7 } var first = Parent { child: child } var second = Parent { child: child } free(first) int result = second.child.value free(second) free(child) return result }"), 7);
}

#[test]
fn invalid_free_child() {
    match std::env::var("LENCY_CODEGEN_INVALID_FREE").as_deref() {
        Ok("literal") => {
            run("int main() { var text = \"borrowed\" free(text) return 0 }");
        }
        Ok("double") => {
            run("struct Item { int value } int main() { var item = Item { value: 7 } free(item) free(item) return 0 }");
        }
        _ => {}
    }
}

#[test]
fn generated_invalid_free_reports_the_lency_frame() {
    for case in ["literal", "double"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "invalid_free_child", "--nocapture"])
            .env("LENCY_CODEGEN_INVALID_FREE", case)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("invalid free"), "{error}");
        assert!(error.contains("at main (manual_memory.lcy:1:"), "{error}");
    }
}
