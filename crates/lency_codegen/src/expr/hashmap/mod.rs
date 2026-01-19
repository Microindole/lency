//! HashMap Code Generation
//!
//! HashMap 方法调用的代码生成
//!
//! 模块结构：
//! - `ffi`: FFI函数声明
//! - `methods`: 具体方法实现  
//! - `mod`: 公共接口和路由

mod ffi;
mod methods;

pub use methods::{gen_hashmap_extern_call, gen_hashmap_new_call};

/// Check if function name is a hashmap extern function
pub fn is_hashmap_extern(name: &str) -> bool {
    matches!(
        name,
        "hashmap_int_new"
            | "hashmap_int_insert"
            | "hashmap_int_get"
            | "hashmap_int_contains"
            | "hashmap_int_remove"
            | "hashmap_int_len"
            | "hashmap_int_free"
            | "hashmap_string_new"
            | "hashmap_string_insert"
            | "hashmap_string_get"
            | "hashmap_string_contains"
            | "hashmap_string_remove"
            | "hashmap_string_len"
    )
}
