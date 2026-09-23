use super::TypeInferer;
use crate::error::SemanticError;
use crate::type_check::TypeChecker;
use lency_syntax::ast::{Expr, Type};

impl<'a> TypeInferer<'a> {
    /// Infer and validate a call through the single call-checking path.
    /// Keeping argument checks here prevents nested calls from bypassing the
    /// strong-static contract when they appear inside another expression.
    pub(crate) fn infer_call(
        &mut self,
        callee: &mut Expr,
        args: &mut [Expr],
        span: &std::ops::Range<usize>,
    ) -> Result<Type, SemanticError> {
        let mut checker = TypeChecker::new(self.scopes);
        checker.scopes.set_current(self.current_scope);
        checker.check_call(callee, args, span)
    }
}
