//! Semantic Analysis Error Types
//!
//! 语义分析错误定义，遵循 Beryl "Crystal Clear" 哲学，
//! 错误信息必须清晰明了，帮助开发者快速定位问题。

use beryl_syntax::ast::Span;
use thiserror::Error;

/// 语义分析错误
#[derive(Debug, Clone, Error)]
pub enum SemanticError {
    // ============ 名称解析错误 ============
    /// 未定义的变量
    #[error("undefined variable '{name}'")]
    UndefinedVariable { name: String, span: Span },

    /// 未定义的函数
    #[error("undefined function '{name}'")]
    UndefinedFunction { name: String, span: Span },

    /// 未定义的类型
    #[error("undefined type '{name}'")]
    UndefinedType { name: String, span: Span },

    /// 重复定义
    #[error("'{name}' is already defined in this scope")]
    DuplicateDefinition {
        name: String,
        span: Span,
        previous_span: Span,
    },

    // ============ 类型检查错误 ============
    /// 类型不匹配
    #[error("type mismatch: expected '{expected}', found '{found}'")]
    TypeMismatch {
        expected: String,
        found: String,
        span: Span,
    },

    /// 无法推导类型
    #[error("cannot infer type for '{name}', please add type annotation")]
    CannotInferType { name: String, span: Span },

    /// 二元操作类型错误
    #[error("operator '{op}' cannot be applied to types '{left}' and '{right}'")]
    InvalidBinaryOp {
        op: String,
        left: String,
        right: String,
        span: Span,
    },

    /// 一元操作类型错误
    #[error("operator '{op}' cannot be applied to type '{operand}'")]
    InvalidUnaryOp {
        op: String,
        operand: String,
        span: Span,
    },

    // ============ Null Safety 错误 (Beryl 核心特性) ============
    /// 将 null 赋给非空类型
    #[error("cannot assign 'null' to non-nullable type '{ty}'")]
    NullAssignmentToNonNullable { ty: String, span: Span },

    /// 未检查可空类型就直接使用
    #[error("value of type '{ty}' might be null, use 'if != null' check or '?' operator")]
    PossibleNullAccess { ty: String, span: Span },

    // ============ 函数调用错误 ============
    /// 参数数量不匹配
    #[error("function '{name}' expects {expected} arguments, but got {found}")]
    ArgumentCountMismatch {
        name: String,
        expected: usize,
        found: usize,
        span: Span,
    },

    /// 返回类型错误
    #[error("return type mismatch: expected '{expected}', found '{found}'")]
    ReturnTypeMismatch {
        expected: String,
        found: String,
        span: Span,
    },

    /// 缺少返回语句
    #[error("function '{name}' must return a value of type '{ty}'")]
    MissingReturn {
        name: String,
        ty: String,
        span: Span,
    },

    // ============ 类相关错误 ============
    /// 未定义的字段
    #[error("type '{class}' has no field named '{field}'")]
    UndefinedField {
        class: String,
        field: String,
        span: Span,
    },

    /// 未定义的方法
    #[error("type '{class}' has no method named '{method}'")]
    UndefinedMethod {
        class: String,
        method: String,
        span: Span,
    },

    /// 不是类类型
    #[error("'{ty}' is not a class type, cannot access members")]
    NotAClass { ty: String, span: Span },

    /// 不是 Struct 类型
    #[error("'{name}' is not a struct type, cannot use in impl block")]
    NotAStruct { name: String, span: Span },

    /// 不可调用
    #[error("'{ty}' is not callable")]
    NotCallable { ty: String, span: Span },

    /// break 语句不在循环内
    #[error("'break' outside loop")]
    BreakOutsideLoop { span: Span },

    /// continue 语句不在循环内
    #[error("'continue' outside loop")]
    ContinueOutsideLoop { span: Span },

    // ============ 数组相关错误 ============
    /// 数组索引编译期越界
    #[error(
        "array index out of bounds: index {index} is out of bounds for array of length {size}"
    )]
    ArrayIndexOutOfBounds { index: i64, size: usize, span: Span },

    // ============ 泛型相关错误 ============
    /// 泛型参数数量不匹配
    #[error("generic type '{name}' expects {expected} type arguments, but got {found}")]
    GenericArityMismatch {
        name: String,
        expected: usize,
        found: usize,
        span: Span,
    },

    /// 不是泛型类型
    #[error("type '{name}' is not generic, but type arguments were provided")]
    NotAGenericType { name: String, span: Span },

    /// 泛型参数必须是具体的
    #[error("generic argument must be a valid type")]
    InvalidGenericArg { span: Span },

    // ============ Trait 相关错误 ============
    /// 未定义的 Trait
    #[error("undefined trait '{name}'")]
    UndefinedTrait { name: String, span: Span },

    /// 缺少 Trait 方法实现
    #[error("missing method '{method_name}' required by trait '{trait_name}'")]
    MissingTraitMethod {
        trait_name: String,
        method_name: String,
        span: Span,
    },

    /// Trait 方法签名不匹配
    #[error("method '{method_name}' signature does not match trait '{trait_name}': expected '{expected}', found '{found}'")]
    TraitMethodSignatureMismatch {
        trait_name: String,
        method_name: String,
        expected: String,
        found: String,
        span: Span,
    },

    // ============ 模式匹配错误 ============
    /// 模式匹配不穷尽
    #[error("pattern not exhaustive. Missing variants: {missing_variants:?}")]
    PatternNotExhaustive {
        missing_variants: Vec<String>,
        span: Span,
    },
}

impl SemanticError {
    /// 获取错误发生的位置
    pub fn span(&self) -> &Span {
        match self {
            Self::UndefinedVariable { span, .. } => span,
            Self::UndefinedFunction { span, .. } => span,
            Self::UndefinedType { span, .. } => span,
            Self::DuplicateDefinition { span, .. } => span,
            Self::TypeMismatch { span, .. } => span,
            Self::CannotInferType { span, .. } => span,
            Self::InvalidBinaryOp { span, .. } => span,
            Self::InvalidUnaryOp { span, .. } => span,
            Self::NullAssignmentToNonNullable { span, .. } => span,
            Self::PossibleNullAccess { span, .. } => span,
            Self::ArgumentCountMismatch { span, .. } => span,
            Self::ReturnTypeMismatch { span, .. } => span,
            Self::MissingReturn { span, .. } => span,
            Self::UndefinedField { span, .. } => span,
            Self::UndefinedMethod { span, .. } => span,
            Self::NotAClass { span, .. } => span,
            Self::NotAStruct { span, .. } => span,
            Self::NotCallable { span, .. } => span,
            Self::BreakOutsideLoop { span } => span,
            Self::ContinueOutsideLoop { span } => span,
            Self::ArrayIndexOutOfBounds { span, .. } => span,
            Self::GenericArityMismatch { span, .. } => span,
            Self::NotAGenericType { span, .. } => span,
            Self::InvalidGenericArg { span, .. } => span,
            Self::UndefinedTrait { span, .. } => span,
            Self::MissingTraitMethod { span, .. } => span,
            Self::TraitMethodSignatureMismatch { span, .. } => span,
            Self::PatternNotExhaustive { span, .. } => span,
        }
    }
}
