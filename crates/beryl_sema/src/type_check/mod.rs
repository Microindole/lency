//! Type Checking
//!
//! 类型检查模块，验证程序的类型正确性。
//! 遵循 Beryl "Safety by Default" 哲学：严格的类型检查，拒绝隐式错误。

use crate::error::SemanticError;
use crate::scope::ScopeStack;
use crate::type_infer::TypeInferer;
use beryl_syntax::ast::{Decl, Expr, Program, Stmt, Type};

pub mod decl;
pub mod expr;
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
    pub fn check(&mut self, program: &mut Program) -> Result<(), Vec<SemanticError>> {
        for decl in &mut program.decls {
            self.check_decl(decl);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// 检查声明 (delegate to module)
    pub fn check_decl(&mut self, decl: &mut Decl) {
        decl::check_decl(self, decl);
    }

    /// 检查语句 (delegate to module)
    pub fn check_stmt(&mut self, stmt: &mut Stmt) {
        stmt::check_stmt(self, stmt);
    }

    /// 检查函数调用 (delegate to expr module)
    pub fn check_call(
        &mut self,
        callee: &mut Expr,
        args: &mut [Expr],
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        expr::check_call(self, callee, args, span)
    }

    /// 推导表达式类型（封装 TypeInferer）
    pub(crate) fn infer_type(&self, expr: &mut Expr) -> Result<Type, SemanticError> {
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
