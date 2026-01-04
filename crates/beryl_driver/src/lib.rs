//! Beryl Compiler Driver
//!
//! 编译器驱动模块，串联所有编译阶段

pub mod error;

pub use error::{CompileError, CompileResult};

use beryl_codegen::compile_to_ir;
use beryl_sema::analyze;
use beryl_syntax::ast::Program;
use beryl_syntax::lexer::Token;
use beryl_syntax::parser::program_parser;
use chumsky::Parser;
use logos::Logos;

/// 编译结果
#[derive(Debug)]
pub struct CompilationOutput {
    /// 生成的 LLVM IR
    pub ir: String,
    /// 警告信息
    pub warnings: Vec<String>,
}

/// 解析源代码
fn parse_source(source: &str) -> CompileResult<Program> {
    // 词法分析
    let tokens: Vec<Token> = Token::lexer(source)
        .spanned()
        .map(|(tok, _span)| tok.map_err(|_| CompileError::LexError("Unknown token".to_string())))
        .collect::<Result<Vec<_>, _>>()?;

    // 语法分析
    let parser = program_parser();
    parser
        .parse(tokens)
        .map_err(|e| CompileError::ParseError(format!("{:?}", e)))
}

/// 编译 Beryl 源代码
///
/// # Arguments
/// * `source` - Beryl 源代码
///
/// # Returns
/// * `Ok(CompilationOutput)` - 编译成功，返回 LLVM IR
/// * `Err(CompileError)` - 编译失败
///
/// # Example
/// ```no_run
/// use beryl_driver::compile;
///
/// let source = "int main() { return 42; }";
/// let output = compile(source).unwrap();
/// println!("{}", output.ir);
/// ```
pub fn compile(source: &str) -> CompileResult<CompilationOutput> {
    // 1. 词法 + 语法分析
    let ast = parse_source(source)?;

    // 2. 语义分析
    let _analysis_result = analyze(&ast).map_err(CompileError::SemanticErrors)?;

    // 3. 代码生成
    let ir = compile_to_ir(&ast, "main", Some(source))?;

    Ok(CompilationOutput {
        ir,
        warnings: Vec::new(),
    })
}

/// 从文件编译
///
/// # Arguments
/// * `path` - 源文件路径
pub fn compile_file(path: &str) -> CompileResult<CompilationOutput> {
    let source = std::fs::read_to_string(path)?;
    compile(&source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_return() {
        let source = r#"
            int main() {
                return 42;
            }
        "#;

        let result = compile(source);
        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());

        let output = result.unwrap();
        // 检查 IR 中包含 main 函数
        assert!(output.ir.contains("@main"));
        // 检查返回 42
        assert!(output.ir.contains("ret i64 42"));
    }

    #[test]
    fn test_compile_with_arithmetic() {
        let source = r#"
            int add(int a, int b) {
                return a + b;
            }

            int main() {
                return add(10, 32);
            }
        "#;

        let result = compile(source);
        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.ir.contains("@add"));
        assert!(output.ir.contains("@main"));
        assert!(output.ir.contains("call"));
    }

    #[test]
    fn test_compile_with_variable() {
        let source = r#"
            int test() {
                var x = 10;
                return x;
            }
        "#;

        let result = compile(source);
        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.ir.contains("alloca"));
        assert!(output.ir.contains("store"));
        assert!(output.ir.contains("load"));
    }

    #[test]
    fn test_compile_undefined_variable_error() {
        let source = r#"
            int main() {
                return x;
            }
        "#;

        let result = compile(source);
        assert!(result.is_err(), "Should fail with undefined variable");

        match result.unwrap_err() {
            CompileError::SemanticErrors(errors) => {
                assert!(!errors.is_empty());
                assert!(format!("{:?}", errors[0]).contains("x"));
            }
            _ => panic!("Expected SemanticErrors"),
        }
    }

    #[test]
    fn test_compile_type_mismatch_error() {
        let source = r#"
            int main() {
                var x: int = "hello";
                return x;
            }
        "#;

        let result = compile(source);
        assert!(result.is_err(), "Should fail with type mismatch");

        match result.unwrap_err() {
            CompileError::SemanticErrors(_) => {
                // Expected
            }
            _ => panic!("Expected SemanticErrors"),
        }
    }
}
