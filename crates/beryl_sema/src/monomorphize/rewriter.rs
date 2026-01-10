//! Generic Rewriter
//!
//! 负责遍历 AST，将 `Type::Generic`（引用）替换为单态化后的具体名称 `Type::Struct`。
//! 例如：`var b: Box<int>` -> `var b: Box__int`。

use crate::monomorphize::mangling::mangle_type;
use beryl_syntax::ast::*;

pub struct Rewriter;

impl Default for Rewriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Rewriter {
    pub fn new() -> Self {
        Self
    }

    pub fn rewrite_type(&self, ty: &Type) -> Type {
        match ty {
            // Box<int> -> Type::Struct("Box__int")
            Type::Generic(_, args) if !args.is_empty() => {
                // 如果是 Generic，我们假设所有的 Generic 都已被单态化并生成了对应的 Struct
                // 这里直接利用 mangling 规则生成新名字
                let mangled_name = mangle_type(ty);
                Type::Struct(mangled_name)
            }

            Type::Vec(inner) => {
                // Vec<T> -> Type::Struct("Vec__T") (假设 Vec 也被视为需要单态化的 struct)
                // 或者说 CodeGen 对 Vec 有特殊处理？
                // 如果 Vec 是特殊处理的（built-in dynamic array），我们可能不需要 rename。
                // 但是如果 Beryl 的 Vec 是通过库实现的 Generic Struct，则需要。
                // 之前的假设是 Vec 是 Primitive。
                // Type::Vec 是 AST 的一种变体。
                // 目前保持 Type::Vec，只是递归 rewrite inner type。
                Type::Vec(Box::new(self.rewrite_type(inner)))
            }

            Type::Array { element_type, size } => Type::Array {
                element_type: Box::new(self.rewrite_type(element_type)),
                size: *size,
            },

            Type::Nullable(inner) => Type::Nullable(Box::new(self.rewrite_type(inner))),

            _ => ty.clone(),
        }
    }

    pub fn rewrite_decl(&self, decl: Decl) -> Decl {
        match decl {
            Decl::Struct {
                span,
                name,
                generic_params,
                fields,
            } => Decl::Struct {
                span,
                name,
                generic_params,
                fields: fields.into_iter().map(|f| self.rewrite_field(f)).collect(),
            },
            Decl::Function {
                span,
                name,
                generic_params,
                params,
                return_type,
                body,
            } => Decl::Function {
                span,
                name,
                generic_params,
                params: params.into_iter().map(|p| self.rewrite_param(p)).collect(),
                return_type: self.rewrite_type(&return_type),
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stmt(stmt))
                    .collect(),
            },
            Decl::ExternFunction {
                span,
                name,
                generic_params,
                params,
                return_type,
            } => Decl::ExternFunction {
                span,
                name,
                generic_params,
                params: params.into_iter().map(|p| self.rewrite_param(p)).collect(),
                return_type: self.rewrite_type(&return_type),
            },
            Decl::Impl {
                span,
                trait_ref,
                type_name,
                generic_params,
                methods,
            } => Decl::Impl {
                span,
                trait_ref,
                type_name,
                generic_params,
                methods: methods.into_iter().map(|m| self.rewrite_decl(m)).collect(),
            },
            // Trait 定义：目前不需要重写，直接保留
            Decl::Trait {
                span,
                name,
                generic_params,
                methods,
            } => Decl::Trait {
                span,
                name,
                generic_params,
                methods,
            },
        }
    }

    fn rewrite_field(&self, field: Field) -> Field {
        Field {
            name: field.name,
            ty: self.rewrite_type(&field.ty),
        }
    }

    fn rewrite_param(&self, param: Param) -> Param {
        Param {
            name: param.name,
            ty: self.rewrite_type(&param.ty),
        }
    }

    fn rewrite_stmt(&self, stmt: Stmt) -> Stmt {
        match stmt {
            Stmt::VarDecl {
                span,
                name,
                ty,
                value,
            } => Stmt::VarDecl {
                span,
                name,
                ty: ty.map(|t| self.rewrite_type(&t)),
                value: self.rewrite_expr(value),
            },
            Stmt::Assignment {
                span,
                target,
                value,
            } => Stmt::Assignment {
                span,
                target: self.rewrite_expr(target),
                value: self.rewrite_expr(value),
            },
            Stmt::Expression(expr) => Stmt::Expression(self.rewrite_expr(expr)),
            Stmt::Block(stmts) => {
                Stmt::Block(stmts.into_iter().map(|s| self.rewrite_stmt(s)).collect())
            }
            Stmt::Return { span, value } => Stmt::Return {
                span,
                value: value.map(|e| self.rewrite_expr(e)),
            },
            Stmt::If {
                span,
                condition,
                then_block,
                else_block,
            } => Stmt::If {
                span,
                condition: self.rewrite_expr(condition),
                then_block: then_block
                    .into_iter()
                    .map(|s| self.rewrite_stmt(s))
                    .collect(),
                else_block: else_block
                    .map(|b| b.into_iter().map(|s| self.rewrite_stmt(s)).collect()),
            },
            Stmt::While {
                span,
                condition,
                body,
            } => Stmt::While {
                span,
                condition: self.rewrite_expr(condition),
                body: body.into_iter().map(|s| self.rewrite_stmt(s)).collect(),
            },
            Stmt::For {
                span,
                init,
                condition,
                update,
                body,
            } => Stmt::For {
                span,
                init: init.map(|s| Box::new(self.rewrite_stmt(*s))),
                condition: condition.map(|e| self.rewrite_expr(e)),
                update: update.map(|s| Box::new(self.rewrite_stmt(*s))),
                body: body.into_iter().map(|s| self.rewrite_stmt(s)).collect(),
            },
            Stmt::ForIn {
                span,
                iterator,
                iterable,
                body,
            } => Stmt::ForIn {
                span,
                iterator,
                iterable: self.rewrite_expr(iterable),
                body: body.into_iter().map(|s| self.rewrite_stmt(s)).collect(),
            },
            Stmt::Break { span } => Stmt::Break { span },
            Stmt::Continue { span } => Stmt::Continue { span },
        }
    }

    fn rewrite_expr(&self, expr: Expr) -> Expr {
        // rewrite_expr needs to rewrite types in Call/StructLiteral if they have explicit types.
        // Currently Expr doesn't hold Types directly except in StructLiteral/VecLiteral potentially.
        // Or if we check Cast.

        let new_kind = match expr.kind {
            ExprKind::Binary(lhs, op, rhs) => ExprKind::Binary(
                Box::new(self.rewrite_expr(*lhs)),
                op,
                Box::new(self.rewrite_expr(*rhs)),
            ),
            ExprKind::Unary(op, operand) => {
                ExprKind::Unary(op, Box::new(self.rewrite_expr(*operand)))
            }
            ExprKind::Call { callee, args } => ExprKind::Call {
                callee: Box::new(self.rewrite_expr(*callee)),
                args: args.into_iter().map(|a| self.rewrite_expr(a)).collect(),
            },
            ExprKind::Get { object, name } => ExprKind::Get {
                object: Box::new(self.rewrite_expr(*object)),
                name,
            },
            ExprKind::SafeGet { object, name } => ExprKind::SafeGet {
                object: Box::new(self.rewrite_expr(*object)),
                name,
            },
            ExprKind::Array(elements) => {
                ExprKind::Array(elements.into_iter().map(|e| self.rewrite_expr(e)).collect())
            }
            ExprKind::Index { array, index } => ExprKind::Index {
                array: Box::new(self.rewrite_expr(*array)),
                index: Box::new(self.rewrite_expr(*index)),
            },
            ExprKind::Print(e) => ExprKind::Print(Box::new(self.rewrite_expr(*e))),

            ExprKind::StructLiteral { type_, fields } => ExprKind::StructLiteral {
                type_: self.rewrite_type(&type_),
                fields: fields
                    .into_iter()
                    .map(|(n, e)| (n, self.rewrite_expr(e)))
                    .collect(),
            },

            ExprKind::VecLiteral(elements) => {
                ExprKind::VecLiteral(elements.into_iter().map(|e| self.rewrite_expr(e)).collect())
            }

            ExprKind::Match {
                value,
                cases,
                default,
            } => ExprKind::Match {
                value: Box::new(self.rewrite_expr(*value)),
                cases: cases
                    .into_iter()
                    .map(|c| MatchCase {
                        pattern: c.pattern,
                        body: Box::new(self.rewrite_expr(*c.body)),
                        span: c.span,
                    })
                    .collect(),
                default: default.map(|e| Box::new(self.rewrite_expr(*e))),
            },

            ExprKind::GenericInstantiation { base, args } => {
                // Rewriting `func::<int>` -> `func__int` (Variable)
                // Assuming base is Variable.
                if let ExprKind::Variable(name) = base.kind {
                    // Rewrite args first (nested generics: func::<Box<int>>)
                    let new_args: Vec<Type> =
                        args.into_iter().map(|t| self.rewrite_type(&t)).collect();
                    // Use mangling logic
                    let dummy_type = Type::Generic(name, new_args);
                    let mangled = mangle_type(&dummy_type);
                    ExprKind::Variable(mangled)
                } else {
                    // Fallback for complex base (should create semantic error earlier)
                    ExprKind::GenericInstantiation {
                        base: Box::new(self.rewrite_expr(*base)),
                        args: args.into_iter().map(|t| self.rewrite_type(&t)).collect(),
                    }
                }
            }

            // Literal, Variable unchanged
            _ => expr.kind,
        };

        Expr {
            kind: new_kind,
            span: expr.span,
        }
    }
}
