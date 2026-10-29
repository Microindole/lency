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

        // TODO: 自举源码完成迁移后移除大写兼容入口；公开类型名为 vec。
        let vec_type = select! { Token::Ident(name) if name == "vec" || name == "Vec" => () }
            .ignore_then(just(Token::Lt))
            .ignore_then(ty.clone())
            .then_ignore(just(Token::Gt))
            .map(|inner| Type::Vec(Box::new(inner)));

        // TODO: 源码完成迁移后移除旧的类型后数组维度兼容入口。
        let array_size = just(Token::LBracket)
            .ignore_then(select! { Token::Int(n) => n })
            .try_map(|n, span| {
                if n > 0 && n <= u32::MAX as i64 {
                    Ok(n as usize)
                } else {
                    Err(Simple::custom(
                        span,
                        "array length must be a positive 32-bit integer",
                    ))
                }
            })
            .then_ignore(just(Token::RBracket));

        // 组合
        let type_without_suffix = choice((vec_type, basic, ident_or_generic))
            .then(array_size.repeated())
            .map(|(base, sizes)| {
                sizes
                    .into_iter()
                    .fold(base, |element_type, size| Type::Array {
                        element_type: Box::new(element_type),
                        size,
                    })
            });

        // 后缀类型修饰符: T? (可空)
        type_without_suffix
            .clone()
            .then(just(Token::Question).or_not())
            .then(
                // 函数类型后缀: int(int, int)
                // 如果后面跟着括号，解析为函数类型
                ty.clone()
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LParen), just(Token::RParen))
                    .or_not(),
            )
            .map(|((t, suffix), func_params)| {
                let base = match suffix {
                    Some(Token::Question) => Type::Nullable(Box::new(t)),
                    _ => t,
                };
                // 如果有函数参数列表，则是函数类型
                if let Some(params) = func_params {
                    Type::Function {
                        param_types: params,
                        return_type: Box::new(base),
                    }
                } else {
                    base
                }
            })
    })
}

/// 解析字段
pub fn field_parser() -> impl Parser<Token, Field, Error = ParserError> + Clone {
    typed_name_parser()
        .then_ignore(just(Token::Semicolon).or_not())
        .map(|(ty, name)| Field { name, ty })
}

/// C 风格声明项；数组维度属于名称后的声明部分。
pub fn typed_name_parser() -> impl Parser<Token, (Type, String), Error = ParserError> + Clone {
    type_parser()
        .then(ident_parser())
        .then(
            select! { Token::Int(n) => n }
                .try_map(|n, span| {
                    if n > 0 && n <= u32::MAX as i64 {
                        Ok(n as usize)
                    } else {
                        Err(Simple::custom(
                            span,
                            "array length must be a positive 32-bit integer",
                        ))
                    }
                })
                .delimited_by(just(Token::LBracket), just(Token::RBracket))
                .repeated(),
        )
        .map(|((ty, name), sizes)| {
            let ty = sizes
                .into_iter()
                .rev()
                .fold(ty, |element_type, size| Type::Array {
                    element_type: Box::new(element_type),
                    size,
                });
            (ty, name)
        })
}

/// 花括号初始化仅在声明初始值位置解析，不改变普通语句块的含义。
pub fn initializer_parser(
    expr: impl Parser<Token, crate::ast::Expr, Error = ParserError> + Clone + 'static,
) -> impl Parser<Token, crate::ast::Expr, Error = ParserError> + Clone {
    recursive(|init| {
        init.separated_by(just(Token::Comma))
            .allow_trailing()
            .delimited_by(just(Token::LBrace), just(Token::RBrace))
            .map_with_span(|values, span| crate::ast::Expr {
                kind: crate::ast::ExprKind::Array(values),
                span,
            })
            .or(expr.clone())
    })
}

/// 解析泛型参数列表: <T, U> 或 <T: Bound, U: Bound>
/// 返回空Vec如果没有泛型参数
/// 解析泛型参数列表: <T, U> 或 <T: Bound, U: Bound>
/// 返回空Vec如果没有泛型参数
pub fn generic_params_parser(
) -> impl Parser<Token, Vec<crate::ast::GenericParam>, Error = ParserError> + Clone {
    // 单个泛型参数: T 或 T: Bound
    let single_param = ident_parser()
        .then(just(Token::Colon).ignore_then(type_parser()).or_not())
        .map_with_span(|(name, bound), span| crate::ast::GenericParam { span, name, bound });

    single_param
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .delimited_by(just(Token::Lt), just(Token::Gt))
        .or_not()
        .map(|opt| opt.unwrap_or_default())
}
