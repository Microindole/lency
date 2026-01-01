//! 二元运算符注册表
//!
//! 使用表驱动方法管理二元运算符的类型签名

use crate::error::SemanticError;
use beryl_syntax::ast::{BinaryOp, Type};

/// 二元运算符签名
///
/// 描述一个二元运算符对特定类型的操作
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOpSignature {
    pub op: BinaryOp,
    pub lhs: Type,
    pub rhs: Type,
    pub result: Type,
}

/// 二元运算符注册表
///
/// 集中管理所有二元运算符的类型规则
pub struct BinaryOpRegistry {
    signatures: Vec<BinaryOpSignature>,
}

impl BinaryOpRegistry {
    /// 创建默认注册表（包含所有内置运算符）
    pub fn new() -> Self {
        let mut registry = Self {
            signatures: Vec::new(),
        };
        registry.register_builtins();
        registry
    }

    /// 注册内置运算符
    fn register_builtins(&mut self) {
        self.register_arithmetic();
        self.register_comparison();
        self.register_logical();
    }

    /// 注册算术运算符
    ///
    /// 包括：+, -, *, /, %
    fn register_arithmetic(&mut self) {
        use BinaryOp::*;
        use Type::*;

        // int op int -> int
        for op in [Add, Sub, Mul, Div, Mod] {
            self.add(op.clone(), Int, Int, Int);
        }

        // float op float -> float
        for op in [Add, Sub, Mul, Div, Mod] {
            self.add(op.clone(), Float, Float, Float);
        }

        // 数值提升: int/float -> float
        for op in [Add, Sub, Mul, Div, Mod] {
            self.add(op.clone(), Int, Float, Float);
            self.add(op.clone(), Float, Int, Float);
        }

        // 字符串连接: string + string -> string
        self.add(Add, String, String, String);
    }

    /// 注册比较运算符
    ///
    /// 包括：==, !=, <, >, <=, >=
    fn register_comparison(&mut self) {
        use BinaryOp::*;
        use Type::*;

        // int 比较
        for op in [Eq, Neq, Lt, Gt, Leq, Geq] {
            self.add(op.clone(), Int, Int, Bool);
        }

        // float 比较
        for op in [Eq, Neq, Lt, Gt, Leq, Geq] {
            self.add(op.clone(), Float, Float, Bool);
        }

        // bool 相等比较
        for op in [Eq, Neq] {
            self.add(op.clone(), Bool, Bool, Bool);
        }

        // string 比较
        for op in [Eq, Neq] {
            self.add(op.clone(), String, String, Bool);
        }
    }

    /// 注册逻辑运算符
    ///
    /// 包括：&&, ||
    fn register_logical(&mut self) {
        use BinaryOp::*;
        use Type::*;

        // bool && bool -> bool
        // bool || bool -> bool
        for op in [And, Or] {
            self.add(op.clone(), Bool, Bool, Bool);
        }
    }

    /// 添加运算符签名
    fn add(&mut self, op: BinaryOp, lhs: Type, rhs: Type, result: Type) {
        self.signatures.push(BinaryOpSignature {
            op,
            lhs,
            rhs,
            result,
        });
    }

    /// 查找运算符签名
    ///
    /// 根据运算符和操作数类型查找结果类型
    ///
    /// # Arguments
    ///
    /// * `op` - 运算符
    /// * `lhs` - 左操作数类型
    /// * `rhs` - 右操作数类型
    /// * `span` - 源码位置（用于错误报告）
    ///
    /// # Returns
    ///
    /// 成功返回结果类型，失败返回语义错误
    pub fn lookup(
        &self,
        op: &BinaryOp,
        lhs: &Type,
        rhs: &Type,
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        // 查找精确匹配
        for sig in &self.signatures {
            if sig.op == *op && sig.lhs == *lhs && sig.rhs == *rhs {
                return Ok(sig.result.clone());
            }
        }

        // 未找到匹配的运算符签名
        Err(SemanticError::InvalidBinaryOp {
            op: format!("{:?}", op),
            left: lhs.to_string(),
            right: rhs.to_string(),
            span: span.clone(),
        })
    }
}

impl Default for BinaryOpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_int() {
        let registry = BinaryOpRegistry::new();

        let result = registry.lookup(&BinaryOp::Add, &Type::Int, &Type::Int, &(0..1));

        assert_eq!(result.unwrap(), Type::Int);
    }

    #[test]
    fn test_arithmetic_float() {
        let registry = BinaryOpRegistry::new();

        let result = registry.lookup(&BinaryOp::Mul, &Type::Float, &Type::Float, &(0..1));

        assert_eq!(result.unwrap(), Type::Float);
    }

    #[test]
    fn test_arithmetic_float_promotion() {
        let registry = BinaryOpRegistry::new();

        // int + float -> float
        let result = registry.lookup(&BinaryOp::Add, &Type::Int, &Type::Float, &(0..1));
        assert_eq!(result.unwrap(), Type::Float);

        // float + int -> float
        let result = registry.lookup(&BinaryOp::Add, &Type::Float, &Type::Int, &(0..1));
        assert_eq!(result.unwrap(), Type::Float);
    }

    #[test]
    fn test_string_concatenation() {
        let registry = BinaryOpRegistry::new();

        let result = registry.lookup(&BinaryOp::Add, &Type::String, &Type::String, &(0..1));

        assert_eq!(result.unwrap(), Type::String);
    }

    #[test]
    fn test_string_subtraction_not_supported() {
        let registry = BinaryOpRegistry::new();

        // string - string 不支持
        let result = registry.lookup(&BinaryOp::Sub, &Type::String, &Type::String, &(0..1));

        assert!(result.is_err());
    }

    #[test]
    fn test_comparison() {
        let registry = BinaryOpRegistry::new();

        let result = registry.lookup(&BinaryOp::Gt, &Type::Int, &Type::Int, &(0..1));
        assert_eq!(result.unwrap(), Type::Bool);

        let result = registry.lookup(&BinaryOp::Leq, &Type::Float, &Type::Float, &(0..1));
        assert_eq!(result.unwrap(), Type::Bool);
    }

    #[test]
    fn test_equality() {
        let registry = BinaryOpRegistry::new();

        // int == int
        let result = registry.lookup(&BinaryOp::Eq, &Type::Int, &Type::Int, &(0..1));
        assert_eq!(result.unwrap(), Type::Bool);

        // bool == bool
        let result = registry.lookup(&BinaryOp::Eq, &Type::Bool, &Type::Bool, &(0..1));
        assert_eq!(result.unwrap(), Type::Bool);

        // string == string
        let result = registry.lookup(&BinaryOp::Eq, &Type::String, &Type::String, &(0..1));
        assert_eq!(result.unwrap(), Type::Bool);
    }

    #[test]
    fn test_logical() {
        let registry = BinaryOpRegistry::new();

        let result = registry.lookup(&BinaryOp::And, &Type::Bool, &Type::Bool, &(0..1));
        assert_eq!(result.unwrap(), Type::Bool);

        let result = registry.lookup(&BinaryOp::Or, &Type::Bool, &Type::Bool, &(0..1));
        assert_eq!(result.unwrap(), Type::Bool);
    }

    #[test]
    fn test_logical_with_non_bool() {
        let registry = BinaryOpRegistry::new();

        // int && int 不支持
        let result = registry.lookup(&BinaryOp::And, &Type::Int, &Type::Int, &(0..1));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_operation() {
        let registry = BinaryOpRegistry::new();

        // bool + bool 不支持
        let result = registry.lookup(&BinaryOp::Add, &Type::Bool, &Type::Bool, &(0..1));
        assert!(result.is_err());
    }
}
