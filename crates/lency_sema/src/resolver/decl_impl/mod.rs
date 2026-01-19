//! Declaration Implementation Resolution
//!
//! 具体的声明解析逻辑拆分模块

pub mod function;
pub mod impl_block;
pub mod types;

pub use function::resolve_function;
pub use impl_block::resolve_impl;
pub use types::{resolve_enum, resolve_struct, resolve_trait};
