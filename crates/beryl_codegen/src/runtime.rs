use inkwell::context::Context;
use inkwell::module::Module;

use inkwell::values::{FunctionValue, PointerValue};
use inkwell::AddressSpace;
use inkwell::IntPredicate;

/// 注入运行时函数 (panic, printf, exit)
pub fn inject_runtime_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    // 1. Declare printf: i32 printf(i8*, ...)
    let i32_type = context.i32_type();
    let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());

    let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
    let printf_func = module.add_function("printf", printf_type, None);

    // 2. Declare exit: void exit(i32)
    let void_type = context.void_type();
    let exit_type = void_type.fn_type(&[i32_type.into()], false);
    let exit_func = module.add_function("exit", exit_type, None);

    // 3. Define __beryl_panic(msg: i8*, line: i32)
    let panic_type = void_type.fn_type(&[i8_ptr_type.into(), i32_type.into()], false);
    let panic_func = module.add_function("__beryl_panic", panic_type, None);

    // Create entry block for panic
    let builder = context.create_builder();
    let entry = context.append_basic_block(panic_func, "entry");
    builder.position_at_end(entry);

    // Get args
    let msg = panic_func.get_nth_param(0).unwrap().into_pointer_value();
    let line = panic_func.get_nth_param(1).unwrap().into_int_value();

    // Call printf("Panic at line %d: %s\n", line, msg)
    // We need a format string global
    let format_str = "Panic at line %d: %s\n";
    let format_global = builder
        .build_global_string_ptr(format_str, "panic_fmt")
        .unwrap();

    // printf args: format, line, msg
    // Note: printf arg order in C is (fmt, ...). Here: fmt, line, msg.
    builder
        .build_call(
            printf_func,
            &[
                format_global.as_pointer_value().into(),
                line.into(),
                msg.into(),
            ],
            "call_printf",
        )
        .unwrap();

    // Call exit(1)
    let exit_code = i32_type.const_int(1, false);
    builder
        .build_call(exit_func, &[exit_code.into()], "call_exit")
        .unwrap();

    // Unreachable
    builder.build_return(None).unwrap();

    panic_func
}

/// 如果 ptr 为空，调用 panic
pub fn gen_null_check<'ctx>(
    context: &'ctx Context,
    builder: &inkwell::builder::Builder<'ctx>,
    panic_func: FunctionValue<'ctx>,
    ptr: PointerValue<'ctx>,
    line: u32,
) {
    let null_ptr = ptr.get_type().const_null();
    let ptr_int = builder
        .build_ptr_to_int(ptr, context.i64_type(), "ptr_int")
        .unwrap();
    let null_int = builder
        .build_ptr_to_int(null_ptr, context.i64_type(), "null_int")
        .unwrap();

    let is_null = builder
        .build_int_compare(IntPredicate::EQ, ptr_int, null_int, "is_null")
        .unwrap();

    let current_block = builder.get_insert_block().unwrap();
    let panic_block = context.append_basic_block(current_block.get_parent().unwrap(), "null_panic");
    let cont_block = context.append_basic_block(current_block.get_parent().unwrap(), "null_cont");

    builder
        .build_conditional_branch(is_null, panic_block, cont_block)
        .unwrap();

    // Panic block
    builder.position_at_end(panic_block);
    let msg_global = builder
        .build_global_string_ptr("Null Reference Error", "null_msg")
        .unwrap();
    let line_val = context.i32_type().const_int(line as u64, false);
    builder
        .build_call(
            panic_func,
            &[msg_global.as_pointer_value().into(), line_val.into()],
            "",
        )
        .unwrap();
    builder.build_unreachable().unwrap();

    // Continue block
    builder.position_at_end(cont_block);
}

/// 数组越界检查 panic_if(index >= len) (unsigned compare covers negative)
pub fn gen_bounds_check<'ctx>(
    context: &'ctx Context,
    builder: &inkwell::builder::Builder<'ctx>,
    panic_func: FunctionValue<'ctx>,
    index: inkwell::values::IntValue<'ctx>,
    len: inkwell::values::IntValue<'ctx>,
    line: u32,
) {
    // Treat as unsigned check: UGE (Unsigned Greater or Equal)
    // If index is negative, it becomes very large positive, >= len.
    // If index >= len, it triggers.
    let is_out_of_bounds = builder
        .build_int_compare(IntPredicate::UGE, index, len, "is_out_of_bounds")
        .unwrap();

    let current_block = builder.get_insert_block().unwrap();
    let panic_block =
        context.append_basic_block(current_block.get_parent().unwrap(), "bounds_panic");
    let cont_block = context.append_basic_block(current_block.get_parent().unwrap(), "bounds_cont");

    builder
        .build_conditional_branch(is_out_of_bounds, panic_block, cont_block)
        .unwrap();

    // Panic block
    builder.position_at_end(panic_block);
    let msg_global = builder
        .build_global_string_ptr("Index Out of Bounds", "bounds_msg")
        .unwrap();
    let line_val = context.i32_type().const_int(line as u64, false);
    builder
        .build_call(
            panic_func,
            &[msg_global.as_pointer_value().into(), line_val.into()],
            "",
        )
        .unwrap();
    builder.build_unreachable().unwrap();

    // Continue block
    builder.position_at_end(cont_block);
}
