use crate::gather::analyze_hir::AnalyzeHirVisitor;

impl<'tcx, 'a> AnalyzeHirVisitor<'tcx, 'a> {
    pub fn observe_call(&mut self, expr: &rustc_hir::Expr) {
        let rustc_hir::ExprKind::Call(func, _args) = expr.kind else {
            panic!("Called observe_call with non-call expression.");
        };

        if let rustc_hir::ExprKind::Path(ref qpath) = func.kind {
            let ldid = expr.hir_id.owner.def_id;
            let typeck = self.tcx.typeck(ldid);
            if let rustc_hir::def::Res::Def(kind, def_id) = typeck.qpath_res(qpath, func.hir_id) {
                // ... and we have type information for it ...

                // FIXME: I have low confidence in this, but for now this resolved a problem with
                // enum and struct tuple constructors which appear as function calls.
                // Given that we are currently ignoring the tracked/untracked boundary,
                // I think this is fine for now. Is there anything different about constructing these
                // types as opposed to calling a function from the perspective of the ATI analysis?
                let is_constructor = matches!(kind, rustc_hir::def::DefKind::Ctor(_, _));
                if !is_constructor && !self.first_pass.is_fn_def_id_tracked(&def_id) {
                    // ... and the function is untracked as self.first_pass never had
                    // the appropriate defid registered for it.

                    // this function call might need to have it's inputs
                    // untupled, and it's output tupled, depending on the type signature.
                    // store all this information in FirstPassInfo.
                    let span = func.span;
                    let ret_ty = typeck.expr_ty(expr);
                    self.first_pass.observe_untracked_fn_call(
                        span,
                        self.tcx.sess.source_map(),
                        ret_ty,
                    );
                }
            }
        } else {
            // FIXME: could an instrumented call have a non-path kind?
            // yes? closures?
        }
    }
}
