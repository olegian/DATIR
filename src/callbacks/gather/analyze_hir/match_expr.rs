//! Defines how the [`AnalyzeHirVisitor`] records information about match expressions.
//!
//! See the top-level comment in [crate::callbacks::gather::analyze_hir] for more information as to
//! why this is necessary.

use crate::callbacks::{gather::analyze_hir::AnalyzeHirVisitor, types::CanBeTupled};

impl<'tcx, 'a> AnalyzeHirVisitor<'tcx, 'a> {
    /// If the match expression targets a `Tagged<T>`, `TaggedRef<T>`, or `TaggedRefMut<T>`,
    /// then the target must be untupled within the instrument compilation so that each 
    /// arm of the match expression can actually be pattern-matched against the target.
    /// 
    /// Note that compound types do not need to be untupled, as their representation is 
    /// unchanged during the transformation.
    pub fn observe_match(&mut self, expr: &rustc_hir::Expr) {
        let rustc_hir::ExprKind::Match(target, _arms, _match_kind) = expr.kind else {
            panic!("Invoked observe_match with non-match expr: {:?}", expr);
        };

        let ldid = target.hir_id.owner.def_id;
        let typeck = self.tcx.typeck(ldid);
        let inner_ty = typeck.expr_ty(target);
        if inner_ty.peel_refs().can_be_tupled() {
            self.first_pass
                .match_on_tagged
                .mark(target.span, self.tcx.sess.source_map());
        }
    }
}
