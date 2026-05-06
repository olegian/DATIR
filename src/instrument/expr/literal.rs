use rustc_ast_pretty::pprust;

use crate::{common::CanBeTupled, instrument::{expr::common, instrument::InstrumentingVisitor}};

/// Invoked whenever the visitor runs into a ExprKind::Lit
///
/// If lit type can be tupled (e.g. integer types):
///       a --> ATI::track(a)
/// type: T --> Tagged<T>
/// If lit cannot be tupled:
///       a --> a
/// type: T --> Tagged<T>
pub fn transform_literal(_visitor: &mut InstrumentingVisitor, lit_expr: &mut rustc_ast::Expr) {
    let rustc_ast::ExprKind::Lit(lit) = &mut lit_expr.kind else {
        panic!(
            "Invoked transform_literal with non-lit expr: {:?}",
            pprust::expr_to_string(lit_expr)
        );
    };

    if !lit.can_be_tupled() {
        return;
    }

    common::tuple(lit_expr);
}
