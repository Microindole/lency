//! Beryl Code Generation
//!
//! LLVM 代码生成模块，将 Beryl AST 转换为 LLVM IR
//!
//! # 架构
//!
//! 模块化设计，遵循开闭原则：
//! - `error.rs` - 错误类型定义
//! - `context.rs` - LLVM 上下文管理
//! - `types.rs` - 类型映射
//! - `expr.rs` - 表达式代码生成
//! - `stmt.rs` - 语句代码生成
//! - `function.rs` - 函数代码生成
//! - `module.rs` - 模块代码生成

pub mod context;
pub mod error;
pub mod expr;
pub mod function;
pub mod module;
pub mod runtime;
pub mod stmt;
pub mod types;

// 重新导出核心类型
pub use context::CodegenContext;
pub use error::{CodegenError, CodegenResult};

use beryl_syntax::ast::Program;
use inkwell::context::Context;
use inkwell::module::Module;
use module::ModuleGenerator;

/// 编译 Beryl 程序为 LLVM IR
///
/// # Arguments
/// * `program` - Beryl AST 程序
/// * `module_name` - 模块名称
///
/// * `source` - 源代码 (可选)
///
/// # Returns
/// * `Ok(String)` - LLVM IR 字符串
/// * `Err(CodegenError)` - 代码生成错误
pub fn compile_to_ir(
    program: &Program,
    module_name: &str,
    source: Option<&str>,
) -> CodegenResult<String> {
    let context = Context::create();
    let mut ctx = CodegenContext::new(&context, module_name, source);

    // 生成代码
    let mut module_gen = ModuleGenerator::new(&mut ctx);
    module_gen.generate(program)?;

    // 验证生成的 IR
    ctx.verify()?;

    // 返回 IR 字符串
    Ok(ctx.print_to_string())
}

/// 编译 Beryl 程序为 LLVM Module（用于进一步处理）
///
/// # Arguments
/// * `program` - Beryl AST 程序
/// * `context` - LLVM Context
/// * `module_name` - 模块名称
/// * `source` - 源代码 (可选)
///
/// # Returns
/// * `Ok(Module)` - LLVM Module
/// * `Err(CodegenError)` - 代码生成错误
pub fn compile_to_module<'ctx>(
    program: &Program,
    context: &'ctx Context,
    module_name: &str,
    source: Option<&str>,
) -> CodegenResult<Module<'ctx>> {
    let mut ctx = CodegenContext::new(context, module_name, source);

    // 生成代码
    let mut module_gen = ModuleGenerator::new(&mut ctx);
    module_gen.generate(program)?;

    // 验证生成的 IR
    ctx.verify()?;

    Ok(ctx.module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use beryl_syntax::ast::*;

    /// 创建简单的测试程序: int main() { return 42; }
    fn make_simple_program() -> Program {
        Program {
            decls: vec![Decl::Function {
                span: 0..30,
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Int,
                body: vec![Stmt::Return {
                    span: 10..20,
                    value: Some(Expr {
                        kind: ExprKind::Literal(Literal::Int(42)),
                        span: 17..19,
                    }),
                }],
            }],
        }
    }

    #[test]
    fn test_compile_simple_program() {
        let program = make_simple_program();
        let result = compile_to_ir(&program, "test_module", None);

        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());

        let ir = result.unwrap();
        println!("Generated IR:\n{}", ir);

        // 检查用户代码被重命名为 __beryl_main
        assert!(ir.contains("define i64 @__beryl_main()"));
        // 检查生成了 main 包装函数
        assert!(ir.contains("define i32 @main()"));
        // 检查返回 42 (in __beryl_main)
        assert!(ir.contains("ret i64 42"));
    }

    #[test]
    fn test_compile_with_arithmetic() {
        // int add(int a, int b) { return a + b; }
        let program = Program {
            decls: vec![Decl::Function {
                span: 0..50,
                name: "add".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        ty: Type::Int,
                    },
                    Param {
                        name: "b".to_string(),
                        ty: Type::Int,
                    },
                ],
                return_type: Type::Int,
                body: vec![Stmt::Return {
                    span: 30..40,
                    value: Some(Expr {
                        kind: ExprKind::Binary(
                            Box::new(Expr {
                                kind: ExprKind::Variable("a".to_string()),
                                span: 37..38,
                            }),
                            BinaryOp::Add,
                            Box::new(Expr {
                                kind: ExprKind::Variable("b".to_string()),
                                span: 41..42,
                            }),
                        ),
                        span: 37..42,
                    }),
                }],
            }],
        };

        let result = compile_to_ir(&program, "test_add", None);
        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());

        let ir = result.unwrap();
        println!("Generated IR:\n{}", ir);

        assert!(ir.contains("define i64 @add(i64"));
        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_compile_with_variable() {
        // int test() { var x = 10; return x; }
        let program = Program {
            decls: vec![Decl::Function {
                span: 0..60,
                name: "test".to_string(),
                params: vec![],
                return_type: Type::Int,
                body: vec![
                    Stmt::VarDecl {
                        span: 15..25,
                        name: "x".to_string(),
                        ty: Some(Type::Int),
                        value: Expr {
                            kind: ExprKind::Literal(Literal::Int(10)),
                            span: 23..25,
                        },
                    },
                    Stmt::Return {
                        span: 26..36,
                        value: Some(Expr {
                            kind: ExprKind::Variable("x".to_string()),
                            span: 33..34,
                        }),
                    },
                ],
            }],
        };

        let result = compile_to_ir(&program, "test_var", None);
        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());

        let ir = result.unwrap();
        println!("Generated IR:\n{}", ir);

        assert!(ir.contains("alloca i64"));
        assert!(ir.contains("store i64 10"));
        assert!(ir.contains("load i64"));
    }
}
