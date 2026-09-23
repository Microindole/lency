//! Lency Compiler Driver
//!
//! 编译器驱动模块，串联所有编译阶段

pub mod error;

pub use error::{CompileError, CompileResult};

use chumsky::Parser;
use lency_codegen::compile_to_ir;
use lency_sema::analyze;
use lency_syntax::ast::Program;
use lency_syntax::lexer::Token;
use lency_syntax::parser::program_parser;
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
/// 解析源代码
fn parse_source(source: &str) -> CompileResult<Program> {
    use chumsky::Stream;

    // 1. 词法分析 (保留 Span)
    let token_iter = Token::lexer(source).spanned().map(|(tok, span)| match tok {
        Ok(t) => (t, span),
        Err(_) => (Token::Error, span),
    });

    let token_vec: Vec<(Token, std::ops::Range<usize>)> = token_iter.collect();
    let len = source.len();

    // 2. 创建 Stream
    // Stream 需要一个迭代器，其中元素是 (Token, Span)
    // 还需要指定 EOF 的 Span (这里用 source.len()..source.len())
    let stream = Stream::from_iter(len..len, token_vec.into_iter());

    // 3. 语法分析
    let parser = program_parser();
    parser.parse(stream).map_err(|e| {
        let details = e
            .into_iter()
            .map(|err| {
                // Simple<Token> has a public span field or method that doesn't need the trait imported if inherent,
                // or if it is imported, it's seemingly not needed according to rustc.
                // Actually Simple struct usually has span()
                let span = err.span();
                let msg = match err.reason() {
                    chumsky::error::SimpleReason::Custom(msg) => msg.clone(),
                    chumsky::error::SimpleReason::Unexpected => {
                        format!(
                            "Unexpected token found: {:?}",
                            err.found()
                                .map(|t| t.to_string())
                                .unwrap_or_else(|| "EOF".to_string())
                        )
                    }
                    chumsky::error::SimpleReason::Unclosed { span: _, delimiter } => {
                        format!("Unclosed delimiter {:?}", delimiter)
                    }
                };

                // Optional: Add "expected" info if useful, but keep it simple
                // let expected = err.expected().map(|t| t.to_string()).collect::<Vec<_>>().join(", ");

                let label = None;
                let mut help = None;

                // Simple heuristic for common errors
                if let chumsky::error::SimpleReason::Unexpected = err.reason() {
                    let expected: Vec<_> = err
                        .expected()
                        .map(|t| {
                            t.as_ref()
                                .map(|tok| tok.to_string())
                                .unwrap_or_else(|| "EOF".to_string())
                        })
                        .collect();

                    if !expected.is_empty() {
                        if expected.len() < 5 {
                            help = Some(format!("expected one of: {}", expected.join(", ")));
                        } else {
                            help = Some(format!(
                                "expected one of: {}, ...",
                                expected
                                    .iter()
                                    .take(4)
                                    .cloned()
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ));
                        }
                    }
                }

                crate::error::ParseErrorDetail {
                    span,
                    message: msg,
                    label,
                    help,
                }
            })
            .collect();
        CompileError::ParseError(details)
    })
}

/// 编译 Lency 源代码
///
/// # Arguments
/// * `source` - Lency 源代码
///
/// # Returns
/// * `Ok(CompilationOutput)` - 编译成功，返回 LLVM IR
/// * `Err(CompileError)` - 编译失败
///
/// # Example
/// ```no_run
/// use lency_driver::compile;
///
/// let source = "int main() { return 42; }";
/// let output = compile(source).unwrap();
/// println!("{}", output.ir);
/// ```
pub fn compile(source: &str) -> CompileResult<CompilationOutput> {
    // 1. 词法 + 语法分析
    let mut ast = parse_source(source)?;

    // 2. 语义分析
    let _analysis_result = analyze(&mut ast).map_err(CompileError::SemanticErrors)?;

    // 3. 单态化 (Generic Monomorphization)
    let mut monomorphizer = lency_monomorph::Monomorphizer::new();
    let monomorphized_ast = monomorphizer.process(ast);

    // 4. 代码生成
    let ir = compile_to_ir(&monomorphized_ast, "main", Some(source))?;

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
                int x = "hello";
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
