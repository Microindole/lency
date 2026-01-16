//! Built-in Extern Functions
//!
//! 内置 extern 函数的注册

use crate::scope::ScopeStack;
use crate::symbol::Symbol;
use lency_syntax::ast::{Span, Type};

/// Register all built-in extern functions
pub fn register_builtins(scopes: &mut ScopeStack) {
    let dummy_span = Span { start: 0, end: 0 };

    // Helper to define extern functions
    let mut define_extern_fn = |name: &str, params: Vec<(&str, Type)>, return_type: Type| {
        let param_vec: Vec<(String, Type)> = params
            .into_iter()
            .map(|(n, t)| (n.to_string(), t))
            .collect();
        let sym = Symbol::Function(crate::symbol::FunctionSymbol::new(
            name.to_string(),
            param_vec,
            return_type,
            dummy_span.clone(),
        ));
        scopes.define(sym).ok();
    };

    // HashMap FFI functions
    define_extern_fn("hashmap_int_new", vec![], Type::Int);
    define_extern_fn(
        "hashmap_int_insert",
        vec![("map", Type::Int), ("key", Type::Int), ("value", Type::Int)],
        Type::Void,
    );
    define_extern_fn(
        "hashmap_int_get",
        vec![("map", Type::Int), ("key", Type::Int)],
        Type::Int,
    );
    define_extern_fn(
        "hashmap_int_contains",
        vec![("map", Type::Int), ("key", Type::Int)],
        Type::Bool,
    );
    define_extern_fn(
        "hashmap_int_remove",
        vec![("map", Type::Int), ("key", Type::Int)],
        Type::Bool,
    );
    define_extern_fn("hashmap_int_len", vec![("map", Type::Int)], Type::Int);

    // Type conversion FFI functions
    define_extern_fn("int_to_string", vec![("n", Type::Int)], Type::String);
    define_extern_fn("float_to_string", vec![("f", Type::Float)], Type::String);
    define_extern_fn("parse_int", vec![("s", Type::String)], Type::Int);
    define_extern_fn("parse_float", vec![("s", Type::String)], Type::Float);

    // File system FFI functions
    define_extern_fn("file_exists", vec![("path", Type::String)], Type::Bool);
    define_extern_fn("is_dir", vec![("path", Type::String)], Type::Bool);
}
