use super::TypeInferer;
use crate::error::SemanticError;
use crate::symbol::Symbol;
use lency_syntax::ast::{Expr, Type, UnaryOp};

impl<'a> TypeInferer<'a> {
    /// 推导二元表达式类型
    pub(crate) fn infer_binary(
        &mut self,
        left: &mut Expr,
        op: &lency_syntax::ast::BinaryOp,
        right: &mut Expr,
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        let left_ty = self.infer(left)?;
        let right_ty = self.infer(right)?;

        // Special handling for Elvis Operator (??)
        if matches!(op, lency_syntax::ast::BinaryOp::Elvis) {
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
        let result = self.binary_ops.lookup(op, &left_ty, &right_ty, span);

        if result.is_ok() {
            return result;
        }

        // Fallback: Check for generic parameters with trait bounds (e.g. T: Comparable)
        if let Type::GenericParam(name) = &left_ty {
            if left_ty == right_ty {
                // Currently restrict to T op T
                if let Some(Symbol::GenericParam(gp)) = self.lookup(name) {
                    if let Some(Type::Struct(tit_name)) = &gp.bound {
                        // Check if trait supports the operator
                        // For now, hardcode standard traits mapping since we don't have operator overloading fully generic mapped yet
                        // Eq -> ==, !=
                        // Comparable -> <, >, <=, >=
                        use lency_syntax::ast::BinaryOp::*;
                        match op {
                            Eq | Neq if tit_name == "Eq" || tit_name == "Comparable" => {
                                // Comparable usually implies Eq
                                return Ok(Type::Bool);
                            }
                            Lt | Leq | Gt | Geq if tit_name == "Comparable" => {
                                return Ok(Type::Bool);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        result
    }

    /// 推导一元表达式类型
    pub(crate) fn infer_unary(
        &mut self,
        op: &UnaryOp,
        operand: &mut Expr,
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        let operand_ty = self.infer(operand)?;

        // 使用运算符表查找
        self.unary_ops.lookup(op, &operand_ty, span)
    }
}
