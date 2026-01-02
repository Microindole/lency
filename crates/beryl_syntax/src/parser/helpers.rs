//! Parser Helper Functions
//!
//! 辅助解析函数：标识符、类型、字段等

use crate::ast::{Field, Type};
use crate::lexer::Token;
use chumsky::prelude::*;

pub type ParserError = Simple<Token>;

/// 解析标识符
pub fn ident_parser() -> impl Parser<Token, String, Error = ParserError> + Clone {
    select! { Token::Ident(ident) => ident }
}

/// 解析类型
pub fn type_parser() -> impl Parser<Token, Type, Error = ParserError> + Clone {
    recursive(|ty| {
        let basic = select! {
            Token::TypeInt => Type::Int,
            Token::TypeFloat => Type::Float,
            Token::TypeString => Type::String,
            Token::TypeBool => Type::Bool,
            Token::TypeVoid => Type::Void,
            Token::Ident(name) => Type::Struct(name),
        };

        // 数组类型: [N]T (Go-style)
        let array_type = just(Token::LBracket)
            .ignore_then(select! { Token::Int(n) => n as usize })
            .then_ignore(just(Token::RBracket))
            .then(ty.clone())
            .map(|(size, element_type)| Type::Array {
                element_type: Box::new(element_type),
                size,
            });

        let base_type = array_type.or(basic);

        // 可空类型: T?
        base_type
            .then(just(Token::Question).or_not())
            .map(|(t, q)| {
                if q.is_some() {
                    Type::Nullable(Box::new(t))
                } else {
                    t
                }
            })
    })
}

/// 解析字段
pub fn field_parser() -> impl Parser<Token, Field, Error = ParserError> + Clone {
    type_parser()
        .then(ident_parser())
        .then_ignore(just(Token::Semicolon).or_not())
        .map(|(ty, name)| Field { name, ty })
}
