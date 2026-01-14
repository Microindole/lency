//! Generic Type Specializer
//!
//! 负责将 AST 中的泛型参数替换为具体类型。
//! 这是一个 AST -> AST 的变换器。

use lency_syntax::ast::*;
use std::collections::HashMap;

pub struct Specializer {
    /// 泛型参数名到具体类型的映射 (e.g., T -> int)
    type_map: HashMap<String, Type>,
}

impl Specializer {
    pub fn new(type_map: HashMap<String, Type>) -> Self {
        Self { type_map }
    }

    /// 特化类型
    pub fn specialize_type(&self, ty: &Type) -> Type {
        match ty {
            // T -> int
            Type::GenericParam(name) => {
                if let Some(concrete_ty) = self.type_map.get(name) {
                    concrete_ty.clone()
                } else {
                    // 如果找不到映射，可能是外层泛型（嵌套函数）或者未被替换
                    // 保持原样
                    Type::GenericParam(name.clone())
                }
            }

            // Parser 将 T 解析为 Type::Struct("T")，所以我们也需要在 Struct 变体中检查替换
            Type::Struct(name) => {
                if let Some(concrete_ty) = self.type_map.get(name) {
                    concrete_ty.clone()
                } else {
                    Type::Struct(name.clone())
                }
            }

            // Box<T> -> Box<int> (注意：这里只替换参数，Box 本身变成 Box__int 是 Rewriter 的工作，或者也可以在这里做？)
            // 单态化通常分两步：1. 替换参数 2. 重命名
            // 这里我们只负责替换 T -> int。重命名 Box<int> -> Box__int 将在后续步骤或同时进行。
            // 实际上，Monomorphizer 会递归处理。
            // 当我们特化 T generic 时，我们产生一个具体的 struct 定义 `struct Box<T=int>`.
            // 在这个新 struct 的 body 里，T 被替换为 int。
            // 而对于 body 里引用的其他泛型 `Vec<T>`，它变成了 `Vec<int>`。
            // 随后 Monomorphizer 会发现需要 `Vec<int>`，于是再去生成 `Vec<int>`.
            // 所以这里只需要替换 Type::GenericParam 即可。
            Type::Generic(name, args) => {
                let new_args = args.iter().map(|arg| self.specialize_type(arg)).collect();
                Type::Generic(name.clone(), new_args)
            }

            Type::Vec(inner) => Type::Vec(Box::new(self.specialize_type(inner))),

            Type::Array { element_type, size } => Type::Array {
                element_type: Box::new(self.specialize_type(element_type)),
                size: *size,
            },

            Type::Nullable(inner) => Type::Nullable(Box::new(self.specialize_type(inner))),

            // 基础类型不变
            _ => ty.clone(),
        }
    }

    /// 特化声明 (Decl)
    /// 注意：我们只特化函数体和结构体字段。Decl 本身的名字修改由 Monomorphizer 处理。
    pub fn specialize_decl(&self, decl: &Decl) -> Decl {
        match decl {
            Decl::Var {
                span,
                name,
                ty,
                value,
            } => Decl::Var {
                span: span.clone(),
                name: name.clone(),
                ty: ty.clone(),       // Should specialize type
                value: value.clone(), // Should specialize expr
            },
            Decl::Import { items, span } => Decl::Import {
                span: span.clone(),
                items: items.clone(),
            },
            Decl::Struct {
                span,
                name,
                generic_params,
                fields,
            } => {
                // Keep generic_params that are NOT in type_map
                // Keep generic_params that are NOT in type_map
                let remaining_params: Vec<GenericParam> = generic_params
                    .iter()
                    .filter(|p| !self.type_map.contains_key(&p.name))
                    .cloned()
                    .collect();

                Decl::Struct {
                    span: span.clone(),
                    name: name.clone(),
                    generic_params: remaining_params,
                    fields: fields.iter().map(|f| self.specialize_field(f)).collect(),
                }
            }
            Decl::Function {
                span,
                name,
                generic_params,
                params,
                return_type,
                body,
            } => {
                let remaining_params: Vec<GenericParam> = generic_params
                    .iter()
                    .filter(|p| !self.type_map.contains_key(&p.name))
                    .cloned()
                    .collect();

                Decl::Function {
                    span: span.clone(),
                    name: name.clone(),
                    generic_params: remaining_params,
                    params: params.iter().map(|p| self.specialize_param(p)).collect(),
                    return_type: self.specialize_type(return_type),
                    body: body.iter().map(|stmt| self.specialize_stmt(stmt)).collect(),
                }
            }
            Decl::ExternFunction {
                span,
                name,
                generic_params,
                params,
                return_type,
            } => {
                let remaining_params: Vec<GenericParam> = generic_params
                    .iter()
                    .filter(|p| !self.type_map.contains_key(&p.name))
                    .cloned()
                    .collect();

                Decl::ExternFunction {
                    span: span.clone(),
                    name: name.clone(),
                    generic_params: remaining_params,
                    params: params.iter().map(|p| self.specialize_param(p)).collect(),
                    return_type: self.specialize_type(return_type),
                }
            }
            Decl::Impl {
                span,
                trait_ref,
                type_name,
                generic_params,
                methods,
            } => {
                let remaining_params: Vec<GenericParam> = generic_params
                    .iter()
                    .filter(|p| !self.type_map.contains_key(&p.name))
                    .cloned()
                    .collect();

                Decl::Impl {
                    span: span.clone(),
                    trait_ref: trait_ref.clone(),
                    type_name: type_name.clone(),
                    generic_params: remaining_params,
                    methods: methods.iter().map(|m| self.specialize_decl(m)).collect(),
                }
            }
            // Trait 定义：目前不需要特化，直接保留
            Decl::Trait {
                span,
                name,
                generic_params,
                methods,
            } => Decl::Trait {
                span: span.clone(),
                name: name.clone(),
                generic_params: generic_params.clone(),
                methods: methods.clone(),
            },
            Decl::Enum {
                span,
                name,
                generic_params,
                variants,
            } => {
                let remaining_params: Vec<GenericParam> = generic_params
                    .iter()
                    .filter(|p| !self.type_map.contains_key(&p.name))
                    .cloned()
                    .collect();

                Decl::Enum {
                    span: span.clone(),
                    name: name.clone(),
                    generic_params: remaining_params,
                    variants: variants
                        .iter()
                        .map(|v| match v {
                            EnumVariant::Unit(n) => EnumVariant::Unit(n.clone()),
                            EnumVariant::Tuple(n, types) => EnumVariant::Tuple(
                                n.clone(),
                                types.iter().map(|t| self.specialize_type(t)).collect(),
                            ),
                        })
                        .collect(),
                }
            }
        }
    }

    fn specialize_field(&self, field: &Field) -> Field {
        Field {
            name: field.name.clone(),
            ty: self.specialize_type(&field.ty),
        }
    }

    fn specialize_param(&self, param: &Param) -> Param {
        Param {
            name: param.name.clone(),
            ty: self.specialize_type(&param.ty),
        }
    }

    fn specialize_stmt(&self, stmt: &Stmt) -> Stmt {
        match stmt {
            Stmt::VarDecl {
                span,
                name,
                ty,
                value,
            } => Stmt::VarDecl {
                span: span.clone(),
                name: name.clone(),
                ty: ty.as_ref().map(|t| self.specialize_type(t)),
                value: self.specialize_expr(value),
            },
            Stmt::Assignment {
                span,
                target,
                value,
            } => Stmt::Assignment {
                span: span.clone(),
                target: self.specialize_expr(target),
                value: self.specialize_expr(value),
            },
            Stmt::Expression(expr) => Stmt::Expression(self.specialize_expr(expr)),
            Stmt::Block(stmts) => {
                Stmt::Block(stmts.iter().map(|s| self.specialize_stmt(s)).collect())
            }
            Stmt::Return { span, value } => Stmt::Return {
                span: span.clone(),
                value: value.as_ref().map(|e| self.specialize_expr(e)),
            },
            Stmt::If {
                span,
                condition,
                then_block,
                else_block,
            } => Stmt::If {
                span: span.clone(),
                condition: self.specialize_expr(condition),
                then_block: then_block.iter().map(|s| self.specialize_stmt(s)).collect(),
                else_block: else_block
                    .as_ref()
                    .map(|b| b.iter().map(|s| self.specialize_stmt(s)).collect()),
            },
            Stmt::While {
                span,
                condition,
                body,
            } => Stmt::While {
                span: span.clone(),
                condition: self.specialize_expr(condition),
                body: body.iter().map(|s| self.specialize_stmt(s)).collect(),
            },
            Stmt::For {
                span,
                init,
                condition,
                update,
                body,
            } => Stmt::For {
                span: span.clone(),
                init: init.as_ref().map(|s| Box::new(self.specialize_stmt(s))),
                condition: condition.as_ref().map(|e| self.specialize_expr(e)),
                update: update.as_ref().map(|s| Box::new(self.specialize_stmt(s))),
                body: body.iter().map(|s| self.specialize_stmt(s)).collect(),
            },
            Stmt::ForIn {
                span,
                iterator,
                iterable,
                body,
            } => Stmt::ForIn {
                span: span.clone(),
                iterator: iterator.clone(),
                iterable: self.specialize_expr(iterable),
                body: body.iter().map(|s| self.specialize_stmt(s)).collect(),
            },
            Stmt::Break { span } => Stmt::Break { span: span.clone() },
            Stmt::Continue { span } => Stmt::Continue { span: span.clone() },
        }
    }

    fn specialize_expr(&self, expr: &Expr) -> Expr {
        // 大部分 Expr 只是递归，但有几类需要特殊处理：
        // 1. StructLiteral/VecLiteral/Call: 其中的类型可能需要特化
        // 2. Variable: 这里的 name 只是 string，如果它引用了泛型函数，应该已经被 Parser 决议？
        //    不，Lency 没有泛型推导，所以 Variable 只是名字。
        //    如果它是泛型函数调用，应该在 Call 中处理？
        //    目前 Lency AST 中 Call 只有 callee。没有 generic args。

        let new_kind = match &expr.kind {
            ExprKind::Literal(lit) => ExprKind::Literal(lit.clone()), // Lit 不变
            ExprKind::Unit => ExprKind::Unit,
            ExprKind::Variable(name) => ExprKind::Variable(name.clone()),
            ExprKind::Binary(lhs, op, rhs) => ExprKind::Binary(
                Box::new(self.specialize_expr(lhs)),
                op.clone(),
                Box::new(self.specialize_expr(rhs)),
            ),
            ExprKind::Unary(op, operand) => {
                ExprKind::Unary(op.clone(), Box::new(self.specialize_expr(operand)))
            }
            ExprKind::Call { callee, args } => ExprKind::Call {
                callee: Box::new(self.specialize_expr(callee)),
                args: args.iter().map(|a| self.specialize_expr(a)).collect(),
            },
            ExprKind::Get { object, name } => ExprKind::Get {
                object: Box::new(self.specialize_expr(object)),
                name: name.clone(),
            },
            ExprKind::SafeGet { object, name } => ExprKind::SafeGet {
                object: Box::new(self.specialize_expr(object)),
                name: name.clone(),
            },
            ExprKind::Array(elements) => {
                ExprKind::Array(elements.iter().map(|e| self.specialize_expr(e)).collect())
            }
            ExprKind::Index { array, index } => ExprKind::Index {
                array: Box::new(self.specialize_expr(array)),
                index: Box::new(self.specialize_expr(index)),
            },
            ExprKind::Print(e) => ExprKind::Print(Box::new(self.specialize_expr(e))),

            ExprKind::StructLiteral { type_, fields } => ExprKind::StructLiteral {
                type_: self.specialize_type(type_),
                fields: fields
                    .iter()
                    .map(|(n, e)| (n.clone(), self.specialize_expr(e)))
                    .collect(),
            },

            ExprKind::VecLiteral(elements) => {
                ExprKind::VecLiteral(elements.iter().map(|e| self.specialize_expr(e)).collect())
            }

            ExprKind::GenericInstantiation { base, args } => ExprKind::GenericInstantiation {
                base: Box::new(self.specialize_expr(base)),
                args: args.iter().map(|t| self.specialize_type(t)).collect(),
            },

            ExprKind::Match {
                value,
                cases,
                default,
            } => ExprKind::Match {
                value: Box::new(self.specialize_expr(value)),
                cases: cases
                    .iter()
                    .map(|c| MatchCase {
                        pattern: c.pattern.clone(), // Patterns (Lit) dont have types yet
                        body: Box::new(self.specialize_expr(&c.body)),
                        span: c.span.clone(),
                    })
                    .collect(),
                default: default.as_ref().map(|e| Box::new(self.specialize_expr(e))),
            },
            // Result 相关表达式
            ExprKind::Try(inner) => ExprKind::Try(Box::new(self.specialize_expr(inner))),
            ExprKind::Ok(inner) => ExprKind::Ok(Box::new(self.specialize_expr(inner))),
            ExprKind::Err(inner) => ExprKind::Err(Box::new(self.specialize_expr(inner))),
            // 闭包
            ExprKind::Closure { params, body } => ExprKind::Closure {
                params: params
                    .iter()
                    .map(|p| Param {
                        name: p.name.clone(),
                        ty: self.specialize_type(&p.ty),
                    })
                    .collect(),
                body: Box::new(self.specialize_expr(body)),
            },
            // File I/O intrinsics
            ExprKind::ReadFile(path) => ExprKind::ReadFile(Box::new(self.specialize_expr(path))),
            ExprKind::WriteFile(path, content) => ExprKind::WriteFile(
                Box::new(self.specialize_expr(path)),
                Box::new(self.specialize_expr(content)),
            ),
            // 字符串内置函数 (Sprint 12)
            ExprKind::Len(arg) => ExprKind::Len(Box::new(self.specialize_expr(arg))),
            ExprKind::Trim(arg) => ExprKind::Trim(Box::new(self.specialize_expr(arg))),
            ExprKind::Split(str_arg, delim) => ExprKind::Split(
                Box::new(self.specialize_expr(str_arg)),
                Box::new(self.specialize_expr(delim)),
            ),
            ExprKind::Join(vec_arg, sep) => ExprKind::Join(
                Box::new(self.specialize_expr(vec_arg)),
                Box::new(self.specialize_expr(sep)),
            ),
            ExprKind::Substr(str_arg, start, len) => ExprKind::Substr(
                Box::new(self.specialize_expr(str_arg)),
                Box::new(self.specialize_expr(start)),
                Box::new(self.specialize_expr(len)),
            ),
        };

        Expr {
            kind: new_kind,
            span: expr.span.clone(),
        }
    }
}
