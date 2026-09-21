use crate::error::SemanticError;
use crate::type_infer::TypeInferer;
use lency_syntax::ast::{Expr, ExprKind, Type};

impl<'a> TypeInferer<'a> {
    pub(crate) fn infer_intrinsic(&mut self, expr: &mut Expr) -> Result<Type, SemanticError> {
        match &mut expr.kind {
            // File I/O intrinsics (Sprint 12)
            ExprKind::ReadFile(path) => {
                // 验证 path 是 string
                let path_ty = self.infer(path)?;
                if path_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: path_ty.to_string(),
                        span: path.span.clone(),
                    });
                }
                Ok(Type::String)
            }
            ExprKind::WriteFile(path, content) => {
                // 验证 path 和 content 都是 string
                let path_ty = self.infer(path)?;
                let content_ty = self.infer(content)?;
                if path_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: path_ty.to_string(),
                        span: path.span.clone(),
                    });
                }
                if content_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: content_ty.to_string(),
                        span: content.span.clone(),
                    });
                }
                Ok(Type::Void)
            }
            // 字符串内置函数 (Sprint 12)
            ExprKind::Len(arg) => {
                // len(string) -> int
                let arg_ty = self.infer(arg)?;
                if arg_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: arg_ty.to_string(),
                        span: arg.span.clone(),
                    });
                }
                Ok(Type::Int)
            }
            ExprKind::Trim(arg) => {
                // trim(string) -> string
                let arg_ty = self.infer(arg)?;
                if arg_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: arg_ty.to_string(),
                        span: arg.span.clone(),
                    });
                }
                Ok(Type::String)
            }
            ExprKind::Split(str_arg, delim) => {
                // split(string, string) -> Vec<string>
                let str_ty = self.infer(str_arg)?;
                let delim_ty = self.infer(delim)?;
                if str_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: str_ty.to_string(),
                        span: str_arg.span.clone(),
                    });
                }
                if delim_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: delim_ty.to_string(),
                        span: delim.span.clone(),
                    });
                }
                Ok(Type::Vec(Box::new(Type::String)))
            }
            ExprKind::Join(vec_arg, sep) => {
                // join(Vec<string>, string) -> string
                let vec_ty = self.infer(vec_arg)?;
                let sep_ty = self.infer(sep)?;
                // 检查是否为 Vec<string>
                match &vec_ty {
                    Type::Vec(inner) if **inner == Type::String => {}
                    _ => {
                        return Err(SemanticError::TypeMismatch {
                            expected: "Vec<string>".to_string(),
                            found: vec_ty.to_string(),
                            span: vec_arg.span.clone(),
                        });
                    }
                }
                if sep_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: sep_ty.to_string(),
                        span: sep.span.clone(),
                    });
                }
                Ok(Type::String)
            }
            ExprKind::Substr(str_arg, start, len_arg) => {
                // substr(string, int, int) -> string
                let str_ty = self.infer(str_arg)?;
                let start_ty = self.infer(start)?;
                let len_ty = self.infer(len_arg)?;
                if str_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: str_ty.to_string(),
                        span: str_arg.span.clone(),
                    });
                }
                if start_ty != Type::Int {
                    return Err(SemanticError::TypeMismatch {
                        expected: "int".to_string(),
                        found: start_ty.to_string(),
                        span: start.span.clone(),
                    });
                }
                if len_ty != Type::Int {
                    return Err(SemanticError::TypeMismatch {
                        expected: "int".to_string(),
                        found: len_ty.to_string(),
                        span: len_arg.span.clone(),
                    });
                }
                Ok(Type::String)
            }
            ExprKind::CharToString(arg) => {
                // char_to_string(int) -> string
                let arg_ty = self.infer(arg)?;
                if arg_ty != Type::Int {
                    return Err(SemanticError::TypeMismatch {
                        expected: "int".to_string(),
                        found: arg_ty.to_string(),
                        span: arg.span.clone(),
                    });
                }
                Ok(Type::String)
            }
            ExprKind::Panic(arg) => {
                // panic(string) -> void (never returns)
                let arg_ty = self.infer(arg)?;
                if arg_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: arg_ty.to_string(),
                        span: arg.span.clone(),
                    });
                }
                Ok(Type::Void)
            }
            ExprKind::Format(template, args) => {
                // format(string, Vec<string>) -> string
                let template_ty = self.infer(template)?;
                if template_ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: "string".to_string(),
                        found: template_ty.to_string(),
                        span: template.span.clone(),
                    });
                }
                let args_ty = self.infer(args)?;
                match &args_ty {
                    Type::Vec(inner) if **inner == Type::String => {}
                    _ => {
                        return Err(SemanticError::TypeMismatch {
                            expected: "Vec<string>".to_string(),
                            found: args_ty.to_string(),
                            span: args.span.clone(),
                        });
                    }
                }
                Ok(Type::String)
            }
            _ => unreachable!("Not an intrinsic expression"),
        }
    }
}
