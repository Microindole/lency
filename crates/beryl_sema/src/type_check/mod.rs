//! Type Checking
//!
//! 类型检查模块，验证程序的类型正确性。
//! 遵循 Beryl "Safety by Default" 哲学：严格的类型检查，拒绝隐式错误。

use crate::error::SemanticError;
use crate::scope::ScopeStack;
use crate::symbol::Symbol;
use crate::type_infer::{is_compatible, TypeInferer};
use beryl_syntax::ast::{Decl, Expr, ExprKind, Program, Stmt, Type};

pub mod decl;
pub mod stmt;

/// 类型检查器
pub struct TypeChecker<'a> {
    pub(crate) scopes: &'a mut ScopeStack,
    pub(crate) errors: Vec<SemanticError>,
    /// 当前函数的返回类型（用于检查 return 语句）
    pub(crate) current_return_type: Option<Type>,
    /// 下一个要处理的子作用域索引（用于同步作用域遍历）
    pub(crate) next_child_index: usize,
    /// 当前循环嵌套深度
    pub(crate) loop_depth: usize,
}

impl<'a> TypeChecker<'a> {
    pub fn new(scopes: &'a mut ScopeStack) -> Self {
        Self {
            scopes,
            errors: Vec::new(),
            current_return_type: None,
            next_child_index: 0,
            loop_depth: 0,
        }
    }

    /// 检查整个程序
    pub fn check(&mut self, program: &Program) -> Result<(), Vec<SemanticError>> {
        for decl in &program.decls {
            self.check_decl(decl);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// 检查声明 (delegate to module)
    pub fn check_decl(&mut self, decl: &Decl) {
        decl::check_decl(self, decl);
    }

    /// 检查语句 (delegate to module)
    pub fn check_stmt(&mut self, stmt: &Stmt) {
        stmt::check_stmt(self, stmt);
    }

    /// 检查函数调用
    pub fn check_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        // 获取函数符号
        let (func, is_method) = match &callee.kind {
            ExprKind::Variable(name) => match self.scopes.lookup(name) {
                Some(Symbol::Function(f)) => (f.clone(), false),
                Some(_) => {
                    return Err(SemanticError::NotCallable {
                        ty: name.clone(),
                        span: span.clone(),
                    });
                }
                None => {
                    return Err(SemanticError::UndefinedFunction {
                        name: name.clone(),
                        span: span.clone(),
                    });
                }
            },
            ExprKind::Get { object, name } => {
                // 方法调用处理
                let obj_type = self.infer_type(object)?;
                match obj_type {
                    Type::Struct(struct_name) => {
                        // 构建 mangled name: StructName_methodName
                        let mangled_name = format!("{}_{}", struct_name, name);
                        match self.scopes.lookup(&mangled_name) {
                            Some(Symbol::Function(f)) => (f.clone(), true),
                            _ => {
                                return Err(SemanticError::UndefinedMethod {
                                    class: struct_name,
                                    method: name.clone(),
                                    span: span.clone(),
                                });
                            }
                        }
                    }
                    Type::Vec(inner_type) => {
                        // Vec 内置方法处理
                        match name.as_str() {
                            "push" => {
                                // push(val)
                                if args.len() != 1 {
                                    return Err(SemanticError::ArgumentCountMismatch {
                                        name: "push".to_string(),
                                        expected: 1,
                                        found: args.len(),
                                        span: span.clone(),
                                    });
                                }
                                let arg_ty = self.infer_type(&args[0])?;
                                if !is_compatible(&inner_type, &arg_ty) {
                                    return Err(SemanticError::TypeMismatch {
                                        expected: inner_type.to_string(),
                                        found: arg_ty.to_string(),
                                        span: args[0].span.clone(),
                                    });
                                }
                                return Ok(Type::Void);
                            }
                            "pop" => {
                                // pop() -> T
                                if !args.is_empty() {
                                    return Err(SemanticError::ArgumentCountMismatch {
                                        name: "pop".to_string(),
                                        expected: 0,
                                        found: args.len(),
                                        span: span.clone(),
                                    });
                                }
                                return Ok(*inner_type);
                            }
                            "len" => {
                                // len() -> int
                                if !args.is_empty() {
                                    return Err(SemanticError::ArgumentCountMismatch {
                                        name: "len".to_string(),
                                        expected: 0,
                                        found: args.len(),
                                        span: span.clone(),
                                    });
                                }
                                return Ok(Type::Int);
                            }
                            "get" => {
                                // get(index) -> T
                                if args.len() != 1 {
                                    return Err(SemanticError::ArgumentCountMismatch {
                                        name: "get".to_string(),
                                        expected: 1,
                                        found: args.len(),
                                        span: span.clone(),
                                    });
                                }
                                let arg_ty = self.infer_type(&args[0])?;
                                if !is_compatible(&Type::Int, &arg_ty) {
                                    return Err(SemanticError::TypeMismatch {
                                        expected: "int".to_string(),
                                        found: arg_ty.to_string(),
                                        span: args[0].span.clone(),
                                    });
                                }
                                return Ok(*inner_type);
                            }
                            "set" => {
                                // set(index, val) -> void
                                if args.len() != 2 {
                                    return Err(SemanticError::ArgumentCountMismatch {
                                        name: "set".to_string(),
                                        expected: 2,
                                        found: args.len(),
                                        span: span.clone(),
                                    });
                                }
                                let index_ty = self.infer_type(&args[0])?;
                                if !is_compatible(&Type::Int, &index_ty) {
                                    return Err(SemanticError::TypeMismatch {
                                        expected: "int".to_string(),
                                        found: index_ty.to_string(),
                                        span: args[0].span.clone(),
                                    });
                                }
                                let val_ty = self.infer_type(&args[1])?;
                                if !is_compatible(&inner_type, &val_ty) {
                                    return Err(SemanticError::TypeMismatch {
                                        expected: inner_type.to_string(),
                                        found: val_ty.to_string(),
                                        span: args[1].span.clone(),
                                    });
                                }
                                return Ok(Type::Void);
                            }
                            _ => {
                                return Err(SemanticError::UndefinedMethod {
                                    class: format!("Vec<{}>", inner_type),
                                    method: name.clone(),
                                    span: span.clone(),
                                });
                            }
                        }
                    }
                    _ => {
                        return Err(SemanticError::NotAStruct {
                            name: obj_type.to_string(),
                            span: object.span.clone(),
                        });
                    }
                }
            }
            _ => {
                // 复杂调用表达式暂不支持
                return Ok(Type::Error);
            }
        };

        // 检查参数数量
        // 如果是方法调用，定义中有隐式 this 参数，所以 args.len() + 1 应该等于 params.len()
        let expected_args = if is_method {
            func.params.len() - 1
        } else {
            func.params.len()
        };

        if args.len() != expected_args {
            return Err(SemanticError::ArgumentCountMismatch {
                name: func.name.clone(),
                expected: expected_args,
                found: args.len(),
                span: span.clone(),
            });
        }

        // 检查每个参数类型
        // 检查每个参数类型
        let skip_count = if is_method { 1 } else { 0 };
        let params_iter = func.params.iter().skip(skip_count);

        for (arg, (_, expected_ty)) in args.iter().zip(params_iter) {
            let arg_ty = self.infer_type(arg)?;
            if !is_compatible(expected_ty, &arg_ty) {
                self.errors.push(SemanticError::TypeMismatch {
                    expected: expected_ty.to_string(),
                    found: arg_ty.to_string(),
                    span: arg.span.clone(),
                });
            }
        }

        Ok(func.return_type.clone())
    }

    /// 推导表达式类型（封装 TypeInferer）
    pub(crate) fn infer_type(&self, expr: &Expr) -> Result<Type, SemanticError> {
        let inferer = TypeInferer::new(self.scopes);
        inferer.infer(expr)
    }

    /// 检查代码块是否有返回语句
    pub(crate) fn has_return(&self, stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Return { .. } => return true,
                Stmt::If {
                    then_block,
                    else_block,
                    ..
                } => {
                    // 只有两个分支都有 return 才算完整覆盖
                    if self.has_return(then_block) {
                        if let Some(else_stmts) = else_block {
                            if self.has_return(else_stmts) {
                                return true;
                            }
                        }
                    }
                }
                Stmt::Block(inner) => {
                    if self.has_return(inner) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}
