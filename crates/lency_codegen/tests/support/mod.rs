use inkwell::{execution_engine::ExecutionEngine, module::Module};

pub fn register_trace(engine: &ExecutionEngine<'_>, module: &Module<'_>) {
    for (name, address) in [
        (
            "lency_alloc",
            lency_runtime::memory::lency_alloc as *const () as usize,
        ),
        (
            "lency_free",
            lency_runtime::memory::lency_free as *const () as usize,
        ),
        (
            "lency_trace_enter",
            lency_runtime::trace::lency_trace_enter as *const () as usize,
        ),
        (
            "lency_trace_location",
            lency_runtime::trace::lency_trace_location as *const () as usize,
        ),
        (
            "lency_trace_leave",
            lency_runtime::trace::lency_trace_leave as *const () as usize,
        ),
    ] {
        if let Some(function) = module.get_function(name) {
            engine.add_global_mapping(&function, address);
        }
    }
}
