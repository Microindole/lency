use super::super::TypeInferer;
use crate::error::SemanticError;
use crate::symbol::Symbol;
use lency_syntax::ast::Type;

impl<'a> TypeInferer<'a> {
    /// 推导变量类型
    pub(crate) fn infer_variable_impl(
        &self,
        name: &str,
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        // 1. 优先检查 Flow Analysis Refinements (Smart Casts)
        if let Some(refined_type) = self.scopes.lookup_refinement(name) {
            return Ok(refined_type);
        }

        match self.lookup(name) {
            Some(symbol) => match symbol.ty() {
                Some(ty) => Ok(ty.clone()),
                None => Err(SemanticError::TypeMismatch {
                    expected: "value expression".into(),
                    found: match symbol {
                        Symbol::Function(_) => format!("function name '{name}' without a call"),
                        _ => format!("type name '{name}'"),
                    },
                    span: span.clone(),
                }),
            },
            None => {
                // 尝试隐式 this 访问
                if let Some(this_sym) = self.lookup("this") {
                    if let Some(Type::Struct(struct_name)) = this_sym.ty() {
                        // 查找结构体定义
                        if let Some(crate::symbol::Symbol::Struct(struct_def)) =
                            self.lookup(struct_name)
                        {
                            if let Some(field) = struct_def.get_field(name) {
                                return Ok(field.ty.clone());
                            }
                        }
                    }
                }

                Err(SemanticError::UndefinedVariable {
                    name: name.to_string(),
                    span: span.clone(),
                })
            }
        }
    }
}
