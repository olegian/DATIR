use rustc_ast_pretty::pprust;

use crate::{common, instrument::instrument::InstrumentingVisitor};

/// Invoked whenever the visitor runs into a ExprKind::AddrOf
pub fn transform_addr_of(visitor: &mut InstrumentingVisitor, addr_of_expr: &mut rustc_ast::Expr) {
    let rustc_ast::ExprKind::AddrOf(_, mutbl, referent) = &mut addr_of_expr.kind else {
        panic!(
            "Invoked transform_addr_of with non addr-of expression: {:?}",
            pprust::expr_to_string(addr_of_expr)
        );
    };

    // This reference is taken after indexing a array/slice with a range.
    // It would be nice to do this on the Index expression itself,
    // however we need to know whether this is a mutable ref or a shared ref
    // to know which method to dispatch.
    if visitor
        .first_pass
        .is_span_index_by_range(referent.span, visitor.psess.source_map())
    {
        let rustc_ast::ExprKind::Index(idx_recv, idx_expr, _) = &referent.kind else {
            panic!(
                "First pass identified {:?} as the span of a index-by-range, yet \
                 second pass found a non-index expression: {:?}",
                &referent.span,
                pprust::expr_to_string(referent)
            );
        };
        let mut_str = if mutbl.is_mut() { "_mut" } else { "" };
        let recv_src = pprust::expr_to_string(idx_recv);
        let idx_src = pprust::expr_to_string(idx_expr);
        let code = format!("{recv_src}.subslice{mut_str}({idx_src})");
        *addr_of_expr = common::parse_expr(visitor.psess, code);
        return;
    }

    // LOW CONFIDENC EON THIS LOOK AT IT AGAIN LATER
    if visitor
        .first_pass
        .is_span_ref_to_tupleable_ty(addr_of_expr.span, visitor.psess.source_map())
    {
        // need to transform to (addr_of).as_tagged_ref()
        let mut_str = if mutbl.is_mut() { "_mut" } else { "" };
        addr_of_expr.kind = rustc_ast::ExprKind::MethodCall(Box::new(rustc_ast::MethodCall {
            seg: rustc_ast::PathSegment::from_ident(rustc_span::Ident::from_str(&format!(
                "as_tagged_ref{mut_str}"
            ))),
            receiver: Box::new(addr_of_expr.clone()), // gross clone get rid of this later
            args: [].into(),
            span: rustc_span::DUMMY_SP,
        }));
    }
}
