//! Generic Type Specializer
//!
//! 负责将 AST 中的泛型参数替换为具体类型。
//! 这是一个 AST -> AST 的变换器。

use beryl_syntax::ast::*;
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
            Decl::Struct {
                span,
                name,
                generic_params,
                fields,
            } => {
                // Keep generic_params that are NOT in type_map
                let remaining_params: Vec<String> = generic_params
                    .iter()
                    .filter(|p| !self.type_map.contains_key(*p))
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
                let remaining_params: Vec<String> = generic_params
                    .iter()
                    .filter(|p| !self.type_map.contains_key(*p))
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
                let remaining_params: Vec<String> = generic_params
                    .iter()
                    .filter(|p| !self.type_map.contains_key(*p))
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
                type_name,
                generic_params,
                methods,
            } => {
                let remaining_params: Vec<String> = generic_params
                    .iter()
                    .filter(|p| !self.type_map.contains_key(*p))
                    .cloned()
                    .collect();

                Decl::Impl {
                    span: span.clone(),
                    type_name: type_name.clone(),
                    generic_params: remaining_params,
                    methods: methods.iter().map(|m| self.specialize_decl(m)).collect(),
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
        //    不，Beryl 没有泛型推导，所以 Variable 只是名字。
        //    如果它是泛型函数调用，应该在 Call 中处理？
        //    目前 Beryl AST 中 Call 只有 callee。没有 generic args。

        let new_kind = match &expr.kind {
            ExprKind::Literal(lit) => ExprKind::Literal(lit.clone()), // Lit 不变
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

            ExprKind::StructLiteral { type_name, fields } => {
                // StructLiteral 的 type_name 只是 String。
                // 如果这是一个泛型 Struct 的实例化（在代码中如何表示？）
                // Parser 可能会把 `Box<int> { ... }` 解析为...
                // 现有的 Parser 只支持 Ident 作为 type_name。
                // 这意味着目前只支持 `Box { ... }` (依赖推导?) 或者我们之前的测试代码有问题？
                // 确实，Beryl 语法对于 Struct Literal 似乎只支持简单名。
                // 这是一个潜在的限制。如果 Parser 不支持 `Box<int> { ... }`，那我们无法显式实例化。
                // 但如果 `type_name` 引用的是已经在上文中被特化过的名字（例如通过 typedef?），那没问题。
                // 对于 Phase 3，我们暂时保留这个名字。后续 Rewriter 会将 Generic Usage 改名。
                // 如果 `type_name` 原本是 "Box"，在这里我们不知道它是不是 "Box<int>"。
                // TypeChecker 知道。但在 AST 变换阶段我们只有 AST。

                // 这是一个难点：如果 AST 中没有存储类型参数信息，我们就无法特化 StructLiteral。
                // 除非 TypeChecker 填充了信息。
                // 不过，我们的目标是支持 `var b: Box<int>;` 这样的定义。
                // 至于 Literal，如果支持 `new Box<int>()` 或者是 Constructor Function，那是在 Call 里。

                ExprKind::StructLiteral {
                    type_name: type_name.clone(),
                    fields: fields
                        .iter()
                        .map(|(n, e)| (n.clone(), self.specialize_expr(e)))
                        .collect(),
                }
            }

            ExprKind::VecLiteral(elements) => {
                ExprKind::VecLiteral(elements.iter().map(|e| self.specialize_expr(e)).collect())
            }

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
        };

        Expr {
            kind: new_kind,
            span: expr.span.clone(),
        }
    }
}
