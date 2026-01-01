//! Beryl 类型系统抽象层
//!
//! 遵循开闭原则：对扩展开放，对修改封闭
//!
//! ## 设计理念
//!
//! - **TypeInfo**: 提供统一的类型查询接口
//! - **TypeRegistry**: 集中管理类型信息（为未来扩展预留）
//!
//! ## 使用示例
//!
//! ```rust
//! use beryl_sema::types::TypeInfo;
//! use beryl_syntax::ast::Type;
//!
//! let ty = Type::Int;
//! assert!(ty.is_numeric());
//! assert!(!ty.is_nullable());
//! ```

pub mod info;
pub mod registry;

pub use info::TypeInfo;
pub use registry::TypeRegistry;
