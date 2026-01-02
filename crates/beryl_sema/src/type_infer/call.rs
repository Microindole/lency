use super::TypeInferer;
use crate::error::SemanticError;
use crate::symbol::Symbol;
use beryl_syntax::ast::{Expr, ExprKind, Type};

impl<'a> TypeInferer<'a> {
    /// 推导函数调用类型
    pub(crate) fn infer_call(
        &self,
        callee: &Expr,
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        // 获取被调用者的名称
        if let ExprKind::Variable(name) = &callee.kind {
            match self.lookup(name) {
                Some(Symbol::Function(func)) => Ok(func.return_type.clone()),
                Some(_) => Err(SemanticError::NotCallable {
                    ty: name.clone(),
                    span: span.clone(),
                }),
                None => Err(SemanticError::UndefinedFunction {
                    name: name.clone(),
                    span: span.clone(),
                }),
            }
        } else {
            // 复杂调用表达式（如 obj.method()），暂时返回 Error
            Ok(Type::Error)
        }
    }
}
