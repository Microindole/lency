//! Beryl 运算符系统
//!
//! 使用表驱动方法处理运算符类型检查，遵循开闭原则
//!
//! ## 设计理念
//!
//! - **表驱动**: 用数据结构描述运算符签名，而非硬编码 match
//! - **可扩展**: 添加新运算符只需在注册表中添加条目
//! - **类型安全**: 编译期保证运算符签名正确
//!
//! ## 使用示例
//!
//! ```rust
//! use beryl_sema::operators::BinaryOpRegistry;
//! use beryl_syntax::ast::{BinaryOp, Type};
//!
//! let registry = BinaryOpRegistry::new();
//! let result = registry.lookup(&BinaryOp::Add, &Type::Int, &Type::Int, &(0..1));
//! assert_eq!(result.unwrap(), Type::Int);
//! ```

pub mod binary;
pub mod unary;

pub use binary::{BinaryOpRegistry, BinaryOpSignature};
pub use unary::{UnaryOpRegistry, UnaryOpSignature};
