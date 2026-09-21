//! Intrinsic Function Parsers
//!
//! 内置函数解析器：print, read_file, write_file, len, trim, split, join, substr

use crate::ast::*;
use crate::lexer::Token;
use chumsky::prelude::*;

use super::ParserError;

/// 创建所有内置函数解析器的组合
/// 返回一个能解析任何内置函数调用的 Parser
pub fn intrinsic_parsers<P>(expr: P) -> impl Parser<Token, Expr, Error = ParserError> + Clone
where
    P: Parser<Token, Expr, Error = ParserError> + Clone,
{
    // print(expr)
    let print_expr = just(Token::Print)
        .ignore_then(
            expr.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .map_with_span(|arg, span| Expr {
            kind: ExprKind::Print(Box::new(arg)),
            span,
        });

    // read_file("path") -> string，I/O 失败时 panic
    let read_file_expr = just(Token::ReadFile)
        .ignore_then(
            expr.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .map_with_span(|path, span| Expr {
            kind: ExprKind::ReadFile(Box::new(path)),
            span,
        });

    // write_file("path", "content") -> void，I/O 失败时 panic
    let write_file_expr = just(Token::WriteFile)
        .ignore_then(
            expr.clone()
                .then_ignore(just(Token::Comma))
                .then(expr.clone())
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .map_with_span(|(path, content), span| Expr {
            kind: ExprKind::WriteFile(Box::new(path), Box::new(content)),
            span,
        });

    // len("hello") -> int
    let len_expr = just(Token::Len)
        .ignore_then(
            expr.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .map_with_span(|arg, span| Expr {
            kind: ExprKind::Len(Box::new(arg)),
            span,
        });

    // trim("  hi  ") -> string
    let trim_expr = just(Token::Trim)
        .ignore_then(
            expr.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .map_with_span(|arg, span| Expr {
            kind: ExprKind::Trim(Box::new(arg)),
            span,
        });

    // split("a,b", ",") -> Vec<string>
    let split_expr = just(Token::Split)
        .ignore_then(
            expr.clone()
                .then_ignore(just(Token::Comma))
                .then(expr.clone())
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .map_with_span(|(str_arg, delim), span| Expr {
            kind: ExprKind::Split(Box::new(str_arg), Box::new(delim)),
            span,
        });

    // join(vec, ",") -> string
    let join_expr = just(Token::Join)
        .ignore_then(
            expr.clone()
                .then_ignore(just(Token::Comma))
                .then(expr.clone())
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .map_with_span(|(vec_arg, sep), span| Expr {
            kind: ExprKind::Join(Box::new(vec_arg), Box::new(sep)),
            span,
        });

    // substr("hello", 0, 2) -> string
    let substr_expr = just(Token::Substr)
        .ignore_then(
            expr.clone()
                .then_ignore(just(Token::Comma))
                .then(expr.clone())
                .then_ignore(just(Token::Comma))
                .then(expr.clone())
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .map_with_span(|((str_arg, start), len), span| Expr {
            kind: ExprKind::Substr(Box::new(str_arg), Box::new(start), Box::new(len)),
            span,
        });

    // char_to_string(65) -> "A"
    let char_to_string_expr = just(Token::CharToString)
        .ignore_then(
            expr.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .map_with_span(|arg, span| Expr {
            kind: ExprKind::CharToString(Box::new(arg)),
            span,
        });

    // panic("error message")
    let panic_expr = just(Token::Panic)
        .ignore_then(
            expr.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .map_with_span(|arg, span| Expr {
            kind: ExprKind::Panic(Box::new(arg)),
            span,
        });

    // format("template {}", args_vec) -> string
    let format_expr = just(Token::Format)
        .ignore_then(
            expr.clone()
                .then_ignore(just(Token::Comma))
                .then(expr.clone())
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .map_with_span(|(template, args), span| Expr {
            kind: ExprKind::Format(Box::new(template), Box::new(args)),
            span,
        });

    // 组合所有内置函数解析器
    print_expr
        .or(read_file_expr)
        .or(write_file_expr)
        .or(len_expr)
        .or(trim_expr)
        .or(split_expr)
        .or(join_expr)
        .or(substr_expr)
        .or(char_to_string_expr)
        .or(panic_expr)
        .or(format_expr)
}
