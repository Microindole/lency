//! Code Generation Error Types
//!
//! 代码生成过程中可能出现的错误

use thiserror::Error;

/// 代码生成错误
#[derive(Debug, Error)]
pub enum CodegenError {
    /// 不是函数声明
    #[error("expected function declaration")]
    NotAFunction,

    /// 未定义的变量
    #[error("undefined variable: {0}")]
    UndefinedVariable(String),

    /// 类型不匹配
    #[error("type mismatch in operation")]
    TypeMismatch,

    /// 不支持的表达式
    #[error("unsupported expression")]
    UnsupportedExpression,

    /// 不支持的操作符
    #[error("unsupported operator: {0}")]
    UnsupportedOperator(String),

    /// 不支持的类型
    #[error("unsupported type: {0}")]
    UnsupportedType(String),

    /// 函数未找到
    #[error("function not found: {0}")]
    FunctionNotFound(String),

    /// LLVM 构建错误
    #[error("LLVM build error: {0}")]
    LLVMBuildError(String),

    /// 不支持的特性
    #[error("unsupported feature: {0}")]
    UnsupportedFeature(String),
}

/// 代码生成结果类型
pub type CodegenResult<T> = Result<T, CodegenError>;
