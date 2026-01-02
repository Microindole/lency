//! Declaration Parser
//!
//! 声明解析：函数、类

use super::helpers::{field_parser, ident_parser, type_parser};
use super::stmt::stmt_parser;
use crate::ast::*;
use crate::lexer::Token;
use chumsky::prelude::*;

pub type ParserError = Simple<Token>;

/// 解析声明 (公共接口)
pub fn decl_parser() -> impl Parser<Token, Decl, Error = ParserError> {
    recursive(|_decl| {
        // 函数声明: int add(int a, int b) { ... }
        let func = type_parser()
            .then(ident_parser())
            .then(
                type_parser()
                    .then(ident_parser())
                    .map(|(ty, name)| Param { name, ty })
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .then(
                stmt_parser()
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(
                |(((return_type, name), params), body), span| Decl::Function {
                    span,
                    name,
                    params,
                    return_type,
                    body,
                },
            );

        // 外部函数声明: extern int print(int n);
        let extern_decl = just(Token::Extern)
            .ignore_then(type_parser())
            .then(ident_parser())
            .then(
                type_parser()
                    .then(ident_parser())
                    .map(|(ty, name)| Param { name, ty })
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .then_ignore(just(Token::Semicolon))
            .map_with_span(|((return_type, name), params), span| Decl::ExternFunction {
                span,
                name,
                params,
                return_type,
            });

        // 结构体声明: struct Point { int x int y }
        let struct_decl = just(Token::Struct)
            .ignore_then(ident_parser())
            .then(
                field_parser()
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(|(name, fields), span| Decl::Struct { span, name, fields });

        // impl 块: impl Point { ... }
        let impl_decl = just(Token::Impl)
            .ignore_then(ident_parser())
            .then(
                func.clone()
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(|(type_name, methods), span| Decl::Impl {
                span,
                type_name,
                methods,
            });

        choice((struct_decl, impl_decl, extern_decl, func))
    })
}
