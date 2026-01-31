use super::expr::literal::literal_value_parser;
// Note: ident_parser removed as we now use select! for Ok/Err support
use crate::ast::MatchPattern;
use crate::lexer::Token;
use chumsky::prelude::*;

pub type ParserError = Simple<Token>;

pub fn pattern_parser() -> impl Parser<Token, MatchPattern, Error = ParserError> + Clone {
    recursive(|pat| {
        // Literal Pattern
        let literal = literal_value_parser().map(MatchPattern::Literal);

        // Wildcard Pattern
        let wildcard = just(Token::Underscore).to(MatchPattern::Wildcard);

        // Sprint 15: Helper parser for identifiers or Ok/Err keywords (for Result pattern matching)
        #[allow(clippy::result_large_err)] // Macro-generated code, unavoidable
        let ident_or_result_variant = select! {
            Token::Ident(ident) => ident,
            Token::Ok => "Ok".to_string(),  // Allow Ok as variant name
            Token::Err => "Err".to_string(), // Allow Err as variant name
        };

        // Qualified identifier pattern (Name or Name.Variant or Ok/Err)
        let qualified_ident = ident_or_result_variant.then(
            just(Token::Dot)
                .ignore_then(ident_or_result_variant)
                .or_not(),
        );

        let ident_pat = qualified_ident
            .then(
                pat.separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LParen), just(Token::RParen))
                    .or_not(),
            )
            .map(|((base, suffix), args)| {
                if let Some(suffix) = suffix {
                    // Enum.Variant form
                    MatchPattern::Variant {
                        name: suffix, // We only store Variant Name in MatchPattern::Variant?
                        // Wait, MatchPattern::Variant { name, sub_patterns }.
                        // AST definition: pub enum MatchPattern { Variant { name: String, sub_patterns: Vec<MatchPattern> }, ... }
                        // It seems we lost the 'Enum Name'.
                        // If we parse `OptionInt.Some`, we need to know `OptionInt`.
                        // Does Sema resolve Variant by just name?
                        // Let's check `lency_sema/src/type_infer/control.rs` check_pattern.

                        // check_pattern uses: `target_ty` (the matched value type) to find the Enum.
                        // Then it looks up `name` (variant name) in that Enum.
                        // So `OptionInt.Some(x)` in pattern:
                        // If value is `OptionInt`, we check if `Some` is a variant of `OptionInt``.
                        // We DON'T seemingly use the qualifier `OptionInt` from the pattern in Sema?
                        // BUT, if the user writes `OptionInt.Some`, we should probably verify it matches target type?
                        // Or is the qualifier just syntax sugar for fully qualified access?

                        // If I map `OptionInt.Some` to just `Variant { name: "Some" }`, it works with Sema (Sema ignores the prefix).
                        // Is that acceptable?
                        // Yes, provided Sema infers Enum from the subject expression type.
                        // The user writing `OptionInt.Some` is helpful for readability, but Sema relies on type inference.
                        // However, if we discard `OptionInt`, we can't strict check it.
                        // Ideally we should verify `OptionInt` matches valid type?
                        // But AST `MatchPattern` only has `Variant { name }`.
                        // AST Change needed? Or just ignore prefix?

                        // For Phase 4, let's just ignore the prefix in AST construction but parse it.
                        // Update: `control.rs` check_pattern:
                        // `MatchPattern::Variant { name, ... }`
                        // `if let Some(Symbol::Enum(e)) = self.lookup(enum_name)` (from target_ty).
                        // `e.get_variant(name)`.

                        // So yes, ignoring prefix is fine for now.
                        sub_patterns: args.unwrap_or_default(),
                    }
                } else if let Some(args) = args {
                    MatchPattern::Variant {
                        name: base,
                        sub_patterns: args,
                    }
                } else {
                    MatchPattern::Variable(base)
                }
            });

        choice((wildcard, literal, ident_pat)).boxed()
    })
}
