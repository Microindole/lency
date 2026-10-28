//! 用模式矩阵检查穷尽性，载荷中的字面量不能代表整个枚举项。

use crate::scope::{ScopeId, ScopeStack};
use crate::symbol::Symbol;
use lency_syntax::ast::{Literal, MatchPattern, Type};

struct Constructor {
    name: String,
    fields: Vec<Type>,
    boolean: Option<bool>,
}

fn catch_all(pattern: &MatchPattern) -> bool {
    matches!(pattern, MatchPattern::Wildcard | MatchPattern::Variable(_))
}

fn constructors(scopes: &ScopeStack, scope: ScopeId, ty: &Type) -> Option<Vec<Constructor>> {
    if *ty == Type::Bool {
        return Some(
            [false, true]
                .into_iter()
                .map(|value| Constructor {
                    name: value.to_string(),
                    fields: vec![],
                    boolean: Some(value),
                })
                .collect(),
        );
    }
    let (name, args) = match ty {
        Type::Struct(name) => (name, &[][..]),
        Type::Generic(name, args) => (name, args.as_slice()),
        _ => return None,
    };
    let Some(Symbol::Enum(enumeration)) = scopes.lookup_from(name, scope) else {
        return None;
    };
    let substitutions = enumeration
        .generic_params
        .iter()
        .zip(args)
        .map(|(parameter, argument)| (parameter.name.clone(), argument.clone()))
        .collect();
    let mut result: Vec<_> = enumeration
        .variants
        .iter()
        .map(|(name, fields)| Constructor {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|ty| super::adt::substitute_type(ty, &substitutions))
                .collect(),
            boolean: None,
        })
        .collect();
    result.sort_by(|left, right| left.name.cmp(&right.name));
    Some(result)
}

fn specialize(rows: &[Vec<MatchPattern>], constructor: &Constructor) -> Vec<Vec<MatchPattern>> {
    rows.iter()
        .filter_map(|row| {
            let (first, rest) = row.split_first()?;
            let mut fields = if catch_all(first) {
                vec![MatchPattern::Wildcard; constructor.fields.len()]
            } else {
                match first {
                    MatchPattern::Literal(Literal::Bool(value))
                        if constructor.boolean == Some(*value) =>
                    {
                        vec![]
                    }
                    MatchPattern::Variant { name, sub_patterns }
                        if constructor.boolean.is_none() && *name == constructor.name =>
                    {
                        sub_patterns.clone()
                    }
                    _ => return None,
                }
            };
            fields.extend_from_slice(rest);
            Some(fields)
        })
        .collect()
}

fn exhaustive(
    scopes: &ScopeStack,
    scope: ScopeId,
    types: &[Type],
    rows: &[Vec<MatchPattern>],
) -> bool {
    if rows.is_empty() {
        return false;
    }
    if rows
        .iter()
        .any(|row| row.len() == types.len() && row.iter().all(catch_all))
    {
        return true;
    }
    let Some((first, rest)) = types.split_first() else {
        return !rows.is_empty();
    };
    if let Some(constructors) = constructors(scopes, scope, first) {
        constructors.iter().all(|constructor| {
            let mut fields = constructor.fields.clone();
            fields.extend_from_slice(rest);
            exhaustive(scopes, scope, &fields, &specialize(rows, constructor))
        })
    } else {
        let defaults: Vec<_> = rows
            .iter()
            .filter_map(|row| {
                let (head, tail) = row.split_first()?;
                catch_all(head).then(|| tail.to_vec())
            })
            .collect();
        exhaustive(scopes, scope, rest, &defaults)
    }
}

pub(super) fn missing(
    scopes: &ScopeStack,
    scope: ScopeId,
    ty: &Type,
    patterns: impl Iterator<Item = MatchPattern>,
) -> Vec<String> {
    let rows: Vec<_> = patterns.map(|pattern| vec![pattern]).collect();
    if let Some(constructors) = constructors(scopes, scope, ty) {
        constructors
            .into_iter()
            .filter_map(|constructor| {
                (!exhaustive(
                    scopes,
                    scope,
                    &constructor.fields,
                    &specialize(&rows, &constructor),
                ))
                .then_some(constructor.name)
            })
            .collect()
    } else if exhaustive(scopes, scope, std::slice::from_ref(ty), &rows) {
        vec![]
    } else {
        vec!["_".into()]
    }
}
