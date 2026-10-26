use inkwell::{context::Context, OptimizationLevel};
use std::cell::Cell;

thread_local! {
    static RECEIVER: Cell<*mut i64> = const { Cell::new(std::ptr::null_mut()) };
    static CALLS: Cell<usize> = const { Cell::new(0) };
}

extern "C" fn receiver() -> *mut i64 {
    CALLS.with(|calls| calls.set(calls.get() + 1));
    RECEIVER.with(Cell::get)
}

extern "C" fn call_count() -> i64 {
    CALLS.with(|calls| calls.get() as i64)
}

extern "C" fn unexpected_panic(_: *const u8, _: u32) -> ! {
    std::process::abort()
}

fn run(source: &str) -> (i64, i64, usize) {
    let mut program = lency_syntax::parser::parse(source).expect("valid syntax");
    lency_sema::analyze(&mut program).expect("valid semantics");
    let context = Context::create();
    let module = lency_codegen::compile_to_module(&program, &context, "members", Some(source))
        .expect("valid LLVM");
    let engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    if let Some(function) = module.get_function("receiver") {
        engine.add_global_mapping(&function, receiver as *const () as usize);
    }
    if let Some(function) = module.get_function("call_count") {
        engine.add_global_mapping(&function, call_count as *const () as usize);
    }
    if let Some(function) = module.get_function("lency_panic_at") {
        engine.add_global_mapping(&function, unexpected_panic as *const () as usize);
    }
    let mut field = 0_i64;
    RECEIVER.with(|slot| slot.set(&mut field));
    CALLS.with(|calls| calls.set(0));
    let result = unsafe {
        engine
            .get_function::<unsafe extern "C" fn() -> i64>("__lency_main")
            .unwrap()
            .call()
    };
    RECEIVER.with(|slot| slot.set(std::ptr::null_mut()));
    (result, field, CALLS.with(Cell::get))
}

#[test]
fn member_assignment_evaluates_receiver_once() {
    assert_eq!(run("struct Item { int value } extern Item receiver(); int main() { receiver().value = 7 return 0 }"), (0, 7, 1));
    assert_eq!(run("struct Item { int value } extern Item receiver(); extern int call_count(); int main() { receiver().value = call_count() return 0 }"), (0, 1, 1));
}

#[test]
fn nullable_struct_coalescing_only_evaluates_the_needed_fallback() {
    for initial in ["null", "receiver()"] {
        let source = format!("struct Item {{ int value }} extern Item receiver(); int main() {{ Item? item = {initial} var found = item ?? receiver() found.value = 5 return found.value }}");
        assert_eq!(run(&source), (5, 5, 1));
    }
}

#[test]
fn checked_nullable_receiver_can_be_assigned_and_read() {
    assert_eq!(run("struct Item { int value } extern Item receiver(); int main() { Item? item = receiver() if item != null { item.value = 7 return item.value } return 0 }"), (7, 7, 1));
}

#[test]
fn unnecessary_null_coalescing_keeps_a_valid_module() {
    for expression in ["0 ?? 9", "(2 + 3) ?? 9"] {
        assert_eq!(
            run(&format!("int main() {{ return {expression} }}")).0,
            if expression.starts_with('0') { 0 } else { 5 }
        );
    }
}
