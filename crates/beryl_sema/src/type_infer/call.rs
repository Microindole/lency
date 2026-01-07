use super::TypeInferer;
use crate::error::SemanticError;
use crate::symbol::Symbol;
use beryl_syntax::ast::{Expr, ExprKind, Type};

impl<'a> TypeInferer<'a> {
    /// 推导函数调用类型
    /// 推导函数调用类型
    pub(crate) fn infer_call(
        &self,
        callee: &mut Expr,
        _args: &mut [Expr],
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        match &callee.kind {
            ExprKind::Variable(name) => {
                match self.lookup(name) {
                    Some(Symbol::Function(func)) => Ok(func.return_type.clone()),
                    Some(Symbol::Struct(s)) => {
                        // Constructor
                        Ok(Type::Struct(s.name.clone()))
                    }
                    Some(_) => Err(SemanticError::NotCallable {
                        ty: name.clone(),
                        span: span.clone(),
                    }),
                    None => Err(SemanticError::UndefinedFunction {
                        name: name.clone(),
                        span: span.clone(),
                    }),
                }
            }
            ExprKind::GenericInstantiation {
                base,
                args: type_args,
            } => {
                // 泛型函数调用
                if let ExprKind::Variable(name) = &base.kind {
                    match self.lookup(name) {
                        Some(Symbol::Function(f)) => {
                            // Check generic arg count
                            if f.generic_params.len() != type_args.len() {
                                return Err(SemanticError::ArgumentCountMismatch {
                                    name: format!("{} generic args", name),
                                    expected: f.generic_params.len(),
                                    found: type_args.len(),
                                    span: span.clone(),
                                });
                            }
                            // Subst map
                            let mut map = std::collections::HashMap::new();
                            for (param, arg_ty) in f.generic_params.iter().zip(type_args.iter()) {
                                map.insert(param.name.as_str().to_string(), arg_ty.clone());
                            }
                            // Substitute return type
                            Ok(crate::type_infer::substitute_type(&f.return_type, &map))
                        }
                        _ => Err(SemanticError::NotCallable {
                            ty: "generic instantiation".into(),
                            span: span.clone(),
                        }),
                    }
                } else {
                    Err(SemanticError::NotCallable {
                        ty: "complex generic instantiation".into(),
                        span: span.clone(),
                    })
                }
            }
            _ => {
                // 复杂调用表达式（如 obj.method()），暂时返回 Error
                Ok(Type::Error)
            }
        }
    }
}
