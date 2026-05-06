use crate::{common::CanBeTupled, gather::analyze_hir::AnalyzeHirVisitor};

impl<'tcx, 'a> AnalyzeHirVisitor<'tcx, 'a> {
    pub fn observe_ref(&mut self, expr: &rustc_hir::Expr) {
        let rustc_hir::ExprKind::AddrOf(_, _, referant) = expr.kind else {
            panic!("Invoked observe_ref with non AddrOf expr {:?}", expr);
        };

        let ldid = expr.hir_id.owner.def_id;
        let typeck = self.tcx.typeck(ldid);
        let inner_ty = typeck.expr_ty(referant);

        // Mirror `recursively_tuple_type` in instrument.rs: it wraps
        // `&[T; N]` / `&[T]` as `TaggedRef` *unconditionally* (the
        // outer wrapper carries the container's Id whether or not
        // the element type is itself tupleable). Pass 1 has to flag
        // those AddrOf spans the same way so the call site emits
        // `arr.as_tagged_ref()` to match the function-signature
        // type, otherwise we get a raw `&Tagged<[T; N]>` against a
        // `TaggedRef<[T; N]>` formal.
        let is_tagged_wrapped = inner_ty.can_be_tupled()
            || matches!(
                inner_ty.kind(),
                rustc_middle::ty::Array(..) | rustc_middle::ty::Slice(..)
            );
        if is_tagged_wrapped {
            self.first_pass
                .observe_ref_to_tupleable_ty(expr.span, self.tcx.sess.source_map());
        }
    }
}
