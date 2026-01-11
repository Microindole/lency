//! Declaration Parser
//!
//! 声明解析：函数、类

use super::helpers::{field_parser, generic_params_parser, ident_parser, type_parser};

use super::stmt::stmt_parser;
use crate::ast::*;
use crate::lexer::Token;
use chumsky::prelude::*;

pub type ParserError = Simple<Token>;

/// 解析声明 (公共接口)
pub fn decl_parser() -> impl Parser<Token, Decl, Error = ParserError> {
    let stmt = stmt_parser().boxed();
    recursive(|_decl| {
        // 函数声明: int add(int a, int b) { ... }
        // 泛型函数: T identity<T>(T x) { ... }
        let func = type_parser()
            .then(ident_parser())
            .then(generic_params_parser()) // 解析 <T, U>
            .then(
                type_parser()
                    .then(ident_parser())
                    .map(|(ty, name)| Param { name, ty })
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .then(
                stmt.clone()
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(
                |((((return_type, name), generic_params), params), body), span| Decl::Function {
                    span,
                    name,
                    generic_params,
                    params,
                    return_type,
                    body,
                },
            );

        // 外部函数声明: extern int print(int n);
        let extern_decl = just(Token::Extern)
            .ignore_then(type_parser())
            .then(ident_parser())
            .then(generic_params_parser()) // 解析 <T>
            .then(
                type_parser()
                    .then(ident_parser())
                    .map(|(ty, name)| Param { name, ty })
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .then_ignore(just(Token::Semicolon))
            .map_with_span(|(((return_type, name), generic_params), params), span| {
                Decl::ExternFunction {
                    span,
                    name,
                    generic_params,
                    params,
                    return_type,
                }
            });

        // 结构体声明: struct Point { int x int y }
        // 泛型结构体: struct Box<T> { T value }
        let struct_decl = just(Token::Struct)
            .ignore_then(ident_parser())
            .then(generic_params_parser()) // 解析 <T, U>
            .then(
                field_parser()
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(|((name, generic_params), fields), span| Decl::Struct {
                span,
                name,
                generic_params,
                fields,
            });

        // impl 块: impl Point { ... }
        // 泛型impl: impl<T> Box<T> { ... }
        // Trait实现: impl Greeter for User { ... }
        let impl_decl = just(Token::Impl)
            .ignore_then(generic_params_parser()) // 解析 <T>
            .then(ident_parser()) // 第一个标识符（可能是 Trait 名或 Type 名）
            .then(just(Token::For).ignore_then(ident_parser()).or_not()) // 可选的 "for TypeName"
            .then(
                func.clone()
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(
                |(((generic_params, first_ident), for_type), methods), span| {
                    if let Some(type_name) = for_type {
                        // impl Trait for Type { ... }
                        Decl::Impl {
                            span,
                            trait_ref: Some(first_ident),
                            type_name,
                            generic_params,
                            methods,
                        }
                    } else {
                        // impl Type { ... }
                        Decl::Impl {
                            span,
                            trait_ref: None,
                            type_name: first_ident,
                            generic_params,
                            methods,
                        }
                    }
                },
            );

        // Trait 方法签名: void greet(); 或 bool equals(T other);
        let trait_method = type_parser()
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
            .map(|((return_type, name), params)| TraitMethod {
                name,
                params,
                return_type,
            });

        // Trait 定义: trait Greeter { void greet(); }
        // 泛型Trait: trait Comparable<T> { bool equals(T other); }
        let trait_decl = just(Token::Trait)
            .ignore_then(ident_parser())
            .then(generic_params_parser())
            .then(
                trait_method
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(|((name, generic_params), methods), span| Decl::Trait {
                span,
                name,
                generic_params,
                methods,
            });

        // Enum Variant: Idle 或 Some(T)
        let enum_variant = ident_parser() // Variant Name
            .then(
                type_parser()
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LParen), just(Token::RParen))
                    .or_not(),
            )
            .then_ignore(just(Token::Comma).or_not()) // Optional trailing comma
            .map(|(name, args)| {
                if let Some(types) = args {
                    EnumVariant::Tuple(name, types)
                } else {
                    EnumVariant::Unit(name)
                }
            });

        // Enum 定义: enum Option<T> { Some(T), None }
        let enum_decl = just(Token::Enum)
            .ignore_then(ident_parser())
            .then(generic_params_parser())
            .then(
                enum_variant
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(|((name, generic_params), variants), span| Decl::Enum {
                span,
                name,
                generic_params,
                variants,
            });

        choice((
            enum_decl,
            trait_decl,
            struct_decl,
            impl_decl,
            extern_decl,
            func,
        ))
    })
}
