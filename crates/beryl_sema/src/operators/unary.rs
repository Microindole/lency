//! 一元运算符注册表
//!
//! 使用表驱动方法管理一元运算符的类型签名

use crate::error::SemanticError;
use beryl_syntax::ast::{Type, UnaryOp};

/// 一元运算符签名
///
/// 描述一个一元运算符对特定类型的操作
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOpSignature {
    pub op: UnaryOp,
    pub operand: Type,
    pub result: Type,
}

/// 一元运算符注册表
///
/// 集中管理所有一元运算符的类型规则
pub struct UnaryOpRegistry {
    signatures: Vec<UnaryOpSignature>,
}

impl UnaryOpRegistry {
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
        use Type::*;
        use UnaryOp::*;

        // 负号：-int -> int, -float -> float
        self.add(Neg, Int, Int);
        self.add(Neg, Float, Float);

        // 逻辑非：!bool -> bool
        self.add(Not, Bool, Bool);
    }

    /// 添加运算符签名
    fn add(&mut self, op: UnaryOp, operand: Type, result: Type) {
        self.signatures.push(UnaryOpSignature {
            op,
            operand,
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
    /// * `operand` - 操作数类型
    /// * `span` - 源码位置（用于错误报告）
    ///
    /// # Returns
    ///
    /// 成功返回结果类型，失败返回语义错误
    pub fn lookup(
        &self,
        op: &UnaryOp,
        operand: &Type,
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        // 查找精确匹配
        for sig in &self.signatures {
            if sig.op == *op && sig.operand == *operand {
                return Ok(sig.result.clone());
            }
        }

        // 未找到匹配的运算符签名
        Err(SemanticError::InvalidUnaryOp {
            op: format!("{:?}", op),
            operand: operand.to_string(),
            span: span.clone(),
        })
    }
}

impl Default for UnaryOpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negation_int() {
        let registry = UnaryOpRegistry::new();

        let result = registry.lookup(&UnaryOp::Neg, &Type::Int, &(0..1));
        assert_eq!(result.unwrap(), Type::Int);
    }

    #[test]
    fn test_negation_float() {
        let registry = UnaryOpRegistry::new();

        let result = registry.lookup(&UnaryOp::Neg, &Type::Float, &(0..1));
        assert_eq!(result.unwrap(), Type::Float);
    }

    #[test]
    fn test_logical_not() {
        let registry = UnaryOpRegistry::new();

        let result = registry.lookup(&UnaryOp::Not, &Type::Bool, &(0..1));
        assert_eq!(result.unwrap(), Type::Bool);
    }

    #[test]
    fn test_invalid_negation() {
        let registry = UnaryOpRegistry::new();

        // -string 不支持
        let result = registry.lookup(&UnaryOp::Neg, &Type::String, &(0..1));
        assert!(result.is_err());

        // -bool 不支持
        let result = registry.lookup(&UnaryOp::Neg, &Type::Bool, &(0..1));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_not() {
        let registry = UnaryOpRegistry::new();

        // !int 不支持
        let result = registry.lookup(&UnaryOp::Not, &Type::Int, &(0..1));
        assert!(result.is_err());

        // !string 不支持
        let result = registry.lookup(&UnaryOp::Not, &Type::String, &(0..1));
        assert!(result.is_err());
    }
}
