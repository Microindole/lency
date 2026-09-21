//! Result Code Generation
//!
//! Result<T, E> 相关的过渡代码生成：构造和内置方法

mod constructor;
mod methods;

pub use constructor::{gen_err, gen_ok};
pub use methods::gen_result_builtin_method;
