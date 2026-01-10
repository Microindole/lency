//! Parser Helper Functions
//!
//! 辅助解析函数：标识符、类型、字段等

use crate::ast::{Field, Type};
use crate::lexer::Token;
use chumsky::prelude::*;

pub type ParserError = Simple<Token>;

/// 解析标识符
#[allow(clippy::result_large_err)]
pub fn ident_parser() -> impl Parser<Token, String, Error = ParserError> + Clone {
    select! { Token::Ident(ident) => ident }
}

/// 解析类型
#[allow(clippy::result_large_err)]
pub fn type_parser() -> impl Parser<Token, Type, Error = ParserError> + Clone {
    recursive(|ty| {
        // 基础类型
        let basic = select! {
            Token::TypeInt => Type::Int,
            Token::TypeFloat => Type::Float,
            Token::TypeString => Type::String,
            Token::TypeBool => Type::Bool,
            Token::TypeVoid => Type::Void,
        };

        // 泛型/结构体: Ident 或 Ident<Type, ...>
        let ident_or_generic = ident_parser()
            .then(
                ty.clone()
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::Lt), just(Token::Gt))
                    .or_not(),
            )
            .map(|(name, args)| {
                if let Some(params) = args {
                    Type::Generic(name, params)
                } else {
                    Type::Struct(name)
                }
            });

        // Vec<Type>
        let vec_type = just(Token::Vec)
            .ignore_then(just(Token::Lt))
            .ignore_then(ty.clone())
            .then_ignore(just(Token::Gt))
            .map(|inner| Type::Vec(Box::new(inner)));

        // [N]Type
        let array_type = just(Token::LBracket)
            .ignore_then(select! { Token::Int(n) => n as usize })
            .then_ignore(just(Token::RBracket))
            .then(ty.clone())
            .map(|(size, element_type)| Type::Array {
                element_type: Box::new(element_type),
                size,
            });

        // 组合
        let type_without_nullable = choice((vec_type, array_type, basic, ident_or_generic));

        // 可空类型 Type?
        type_without_nullable
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

/// 解析泛型参数列表: <T, U> 或 <T: Bound, U: Bound>
/// 返回空Vec如果没有泛型参数
/// 注意：当前版本暂时忽略约束（bounds），仅返回参数名称
pub fn generic_params_parser() -> impl Parser<Token, Vec<String>, Error = ParserError> + Clone {
    // 单个泛型参数: T 或 T: Bound
    let single_param = ident_parser()
        .then(just(Token::Colon).ignore_then(type_parser()).or_not())
        .map(|(name, _bound)| name); // TODO: 存储 bound 以支持约束验证

    single_param
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .delimited_by(just(Token::Lt), just(Token::Gt))
        .or_not()
        .map(|opt| opt.unwrap_or_default())
}
