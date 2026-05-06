use crate::{common::CanBeTupled, gather::analyze_hir::AnalyzeHirVisitor};

// A TaggedRef(Mut?)<T> == (&(mut?) Id, &(mut?) T). Nested references will only
// turn the innermost reference into a TaggedRef. In other words, for some tuplable T,
// &&&T becomes &&&Tagged<T> which then becomes &&TaggedRef<T>, to avoid dragging around an
// owned Id. If T is a not a tuplable type (i.e. a user-defined compound type), then
// &T remains &T.
//
// To summarize, for some Non-tupleable primitive type T, tupleable primitive type P,
// and compound type C, we perform one of:
// &N --> &N
// &P --> TR<P>  <-- we are looking for locations where this happens.
// &C --> &C
impl<'tcx, 'a> AnalyzeHirVisitor<'tcx, 'a> {
    pub fn observe_deref(&mut self, expr: &rustc_hir::Expr) {
        let rustc_hir::ExprKind::Unary(rustc_hir::UnOp::Deref, inner) = expr.kind else {
            panic!("Called observe_deref with non-deref unary op: {:?}", expr);
        };

        let ldid = expr.hir_id.owner.def_id;
        let typeck = self.tcx.typeck(ldid);
        let inner_ty = typeck.expr_ty(inner);
        if let rustc_middle::ty::Ref(_, referent, _) = *inner_ty.kind() {
            if referent.can_be_tupled() {
                self.first_pass
                    .observe_tag_stripping_deref(expr.span, self.tcx.sess.source_map());
            }
        }
    }
}
