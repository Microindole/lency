//! Beryl Semantic Analysis
//!
//! 语义分析模块，负责：
//! - 名称解析 (Resolver)
//! - 类型推导 (TypeInferer)
//! - 类型检查 (TypeChecker)
//! - 空安全检查 (NullSafetyChecker)
//!
//! # 设计原则
//!
//! 遵循 Beryl 语言哲学：
//! - **Crystal Clear**: 类型系统透明，错误信息清晰
//! - **Safety by Default**: 默认非空，强制空安全检查
//!
//! 遵循开闭原则 (OCP)：
//! - 新增检查规则只需添加新模块
//! - 核心数据结构稳定不变

pub mod error;
pub mod null_safety;
pub mod operators;
pub mod resolver;
pub mod scope;
pub mod symbol;
pub mod type_check;
pub mod type_infer;
pub mod types;

// 重新导出核心类型
pub use error::SemanticError;
pub use null_safety::NullSafetyChecker;
pub use operators::{BinaryOpRegistry, UnaryOpRegistry};
pub use resolver::Resolver;
pub use scope::{Scope, ScopeId, ScopeKind, ScopeStack};
pub use symbol::{ClassSymbol, FunctionSymbol, Symbol, SymbolId, VariableSymbol};
pub use type_check::TypeChecker;
pub use type_infer::TypeInferer;
pub use types::{TypeInfo, TypeRegistry};

use beryl_syntax::ast::Program;

/// 语义分析结果
#[derive(Debug)]
pub struct AnalysisResult {
    /// 符号表（包含所有定义的符号）
    pub scopes: ScopeStack,
}

/// 分析整个程序
///
/// 按顺序执行三个 Pass：
/// 1. **Resolver**: 收集定义，解析名称引用
/// 2. **TypeChecker**: 类型推导和检查
/// 3. **NullSafetyChecker**: 空安全检查
///
/// # Errors
///
/// 返回所有收集到的语义错误
pub fn analyze(program: &Program) -> Result<AnalysisResult, Vec<SemanticError>> {
    let mut all_errors: Vec<SemanticError> = Vec::new();

    // Pass 1: 名称解析
    let mut resolver = Resolver::new();
    if let Err(errors) = resolver.resolve(program) {
        all_errors.extend(errors);
    }

    // 即使有错误也继续，收集尽可能多的错误信息
    let mut scopes = resolver.into_scopes();

    // Pass 2: 类型检查
    let mut type_checker = TypeChecker::new(&mut scopes);
    if let Err(errors) = type_checker.check(program) {
        all_errors.extend(errors);
    }

    // Pass 3: 空安全检查
    let mut null_checker = NullSafetyChecker::new(&scopes);
    if let Err(errors) = null_checker.check(program) {
        all_errors.extend(errors);
    }

    if all_errors.is_empty() {
        Ok(AnalysisResult { scopes })
    } else {
        Err(all_errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beryl_syntax::ast::*;

    /// 创建一个简单的测试程序
    fn make_simple_program() -> Program {
        // int main() { var x = 10; return x; }
        Program {
            decls: vec![Decl::Function {
                span: 0..50,
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Int,
                body: vec![
                    Stmt::VarDecl {
                        span: 10..20,
                        name: "x".to_string(),
                        ty: Some(Type::Int),
                        value: Expr {
                            kind: ExprKind::Literal(Literal::Int(10)),
                            span: 15..17,
                        },
                    },
                    Stmt::Return {
                        span: 20..30,
                        value: Some(Expr {
                            kind: ExprKind::Variable("x".to_string()),
                            span: 27..28,
                        }),
                    },
                ],
            }],
        }
    }

    #[test]
    fn test_analyze_simple_program() {
        let program = make_simple_program();
        let result = analyze(&program);
        assert!(
            result.is_ok(),
            "Analysis should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_undefined_variable_error() {
        // int main() { return y; }  -- y 未定义
        let program = Program {
            decls: vec![Decl::Function {
                span: 0..30,
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Int,
                body: vec![Stmt::Return {
                    span: 10..20,
                    value: Some(Expr {
                        kind: ExprKind::Variable("y".to_string()),
                        span: 17..18,
                    }),
                }],
            }],
        };

        let result = analyze(&program);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, SemanticError::UndefinedVariable { name, .. } if name == "y")));
    }

    #[test]
    fn test_type_mismatch_error() {
        // int main() { var x: int = "hello"; return x; }
        let program = Program {
            decls: vec![Decl::Function {
                span: 0..50,
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Int,
                body: vec![
                    Stmt::VarDecl {
                        span: 10..30,
                        name: "x".to_string(),
                        ty: Some(Type::Int),
                        value: Expr {
                            kind: ExprKind::Literal(Literal::String("hello".to_string())),
                            span: 20..27,
                        },
                    },
                    Stmt::Return {
                        span: 30..40,
                        value: Some(Expr {
                            kind: ExprKind::Variable("x".to_string()),
                            span: 37..38,
                        }),
                    },
                ],
            }],
        };

        let result = analyze(&program);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, SemanticError::TypeMismatch { .. })));
    }

    #[test]
    fn test_null_safety_error() {
        // void test() { var s: string = null; }  -- null 赋给非空类型
        let program = Program {
            decls: vec![Decl::Function {
                span: 0..50,
                name: "test".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::VarDecl {
                    span: 10..40,
                    name: "s".to_string(),
                    ty: Some(Type::String),
                    value: Expr {
                        kind: ExprKind::Literal(Literal::Null),
                        span: 30..34,
                    },
                }],
            }],
        };

        let result = analyze(&program);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, SemanticError::NullAssignmentToNonNullable { .. })));
    }
}
