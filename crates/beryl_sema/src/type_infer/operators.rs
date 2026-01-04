use super::TypeInferer;
use crate::error::SemanticError;
use beryl_syntax::ast::{Expr, Type, UnaryOp};

impl<'a> TypeInferer<'a> {
    /// 推导二元表达式类型
    pub(crate) fn infer_binary(
        &self,
        left: &Expr,
        op: &beryl_syntax::ast::BinaryOp,
        right: &Expr,
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        let left_ty = self.infer(left)?;
        let right_ty = self.infer(right)?;

        // Special handling for Elvis Operator (??)
        if matches!(op, beryl_syntax::ast::BinaryOp::Elvis) {
            match &left_ty {
                Type::Nullable(inner) => {
                    // Start simple: Right must be compatible with Inner
                    // TODO: Implement proper Lowest Common Supertype (LUB)
                    if crate::type_infer::is_compatible(inner, &right_ty) {
                        return Ok((**inner).clone());
                    } else if crate::type_infer::is_compatible(&right_ty, inner) {
                        return Ok(right_ty);
                    }
                    return Err(SemanticError::TypeMismatch {
                        expected: inner.to_string(),
                        found: right_ty.to_string(),
                        span: right.span.clone(),
                    });
                }
                _ => {
                    // Left is not nullable, so result is left_ty.
                    // Warning could be emitted here (unnecessary elvis).
                    return Ok(left_ty);
                }
            }
        }

        // 使用运算符表查找
        self.binary_ops.lookup(op, &left_ty, &right_ty, span)
    }

    /// 推导一元表达式类型
    pub(crate) fn infer_unary(
        &self,
        op: &UnaryOp,
        operand: &Expr,
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        let operand_ty = self.infer(operand)?;

        // 使用运算符表查找
        self.unary_ops.lookup(op, &operand_ty, span)
    }
}
