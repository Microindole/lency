use inkwell::{context::Context, OptimizationLevel};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
mod support;

static CALLBACK: AtomicUsize = AtomicUsize::new(0);
extern "C" fn callback() -> *const () {
    CALLBACK.load(Ordering::SeqCst) as *const ()
}

fn run(input: i64, null: bool) -> i64 {
    let source = format!("extern int(int) callback();\nint compute(int value) {{ return 42 / value }}\nint main() {{ var function = callback() return function({input}) }}");
    let mut program = lency_syntax::parser::parse(&source).unwrap();
    lency_sema::analyze(&mut program).unwrap();
    let context = Context::create();
    let module =
        lency_codegen::compile_to_module(&program, &context, "callback.lcy", Some(&source))
            .unwrap();
    let engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    support::register_trace(&engine, &module);
    engine.add_global_mapping(
        &module.get_function("callback").unwrap(),
        callback as *const () as usize,
    );
    engine.add_global_mapping(
        &module.get_function("lency_panic_at").unwrap(),
        lency_runtime::lency_panic_at as *const () as usize,
    );
    CALLBACK.store(
        if null {
            0
        } else {
            engine.get_function_address("compute").unwrap()
        },
        Ordering::SeqCst,
    );
    unsafe {
        let main = engine
            .get_function::<unsafe extern "C" fn() -> i64>("__lency_main")
            .unwrap();
        main.call()
    }
}

#[test]
fn indirect_call_child() {
    match std::env::var("LENCY_CALLBACK_CONTRACT").as_deref() {
        Ok("valid") => assert_eq!(run(6, false), 7),
        Ok("zero") => {
            run(0, false);
        }
        Ok("null") => {
            run(6, true);
        }
        _ => {}
    }
}

#[test]
fn indirect_calls_preserve_source_frames_and_check_null_targets() {
    for (case, exit, expected) in [
        ("valid", 0, ""),
        ("zero", 1, "at compute (callback.lcy:2:"),
        ("null", 1, "Null"),
    ] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "indirect_call_child", "--nocapture"])
            .env("LENCY_CALLBACK_CONTRACT", case)
            .output()
            .unwrap();
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(exit), "{case}: {diagnostic}");
        assert!(diagnostic.contains(expected), "{case}: {diagnostic}");
        if exit != 0 {
            assert!(
                diagnostic.contains("at main (callback.lcy:3:"),
                "{diagnostic}"
            );
        }
    }
}
