//! Defines how the [`AnalyzeHirVisitor`] records information about reference-typed expressions.
//!
//! See the top-level comment in [crate::callbacks::gather::analyze_hir] for more information as to
//! why this is necessary.

use crate::{callbacks::gather::analyze_hir::AnalyzeHirVisitor, callbacks::types::CanBeTupled};

impl<'tcx, 'a> AnalyzeHirVisitor<'tcx, 'a> {
    /// If `expr`'s adjusted type is `&T` / `&mut T` with `T` either tupleable or an
    /// array/slice, record the span and original mutability in
    /// [`FirstPassInfo::ref_to_tupleable`](crate::callbacks::gather::first_pass_info::FirstPassInfo::ref_to_tupleable).
    /// Pass 2 uses this to normalize all four post-instrumentation operand shapes
    /// (`Tagged<T>`, `&Tagged<T>`, `TaggedRef<T>`, `TaggedRefMut<T>`) into a uniform
    /// `TaggedRef<T>` / `TaggedRefMut<T>` via `.share()` / `.reborrow()`.
    pub fn observe_ref_normalization(&mut self, expr: &rustc_hir::Expr<'tcx>) {
        let ldid = expr.hir_id.owner.def_id;
        let typeck = self.tcx.typeck(ldid);
        let expr_ty = typeck.expr_ty(expr);

        // Tupleable scalars become `Tagged<T>`, and arrays/slices become
        // `Tagged<[T; N]>` / `TaggedRef<[T]>`. In all of those cases, a `&` / `&mut`
        // of the value normalizes to `TaggedRef` / `TaggedRefMut`.
        let rustc_middle::ty::Ref(_, referent, mutbl) = *expr_ty.kind() else {
            return;
        };
        let is_tagged_wrapped = referent.can_be_tupled()
            || matches!(
                referent.kind(),
                rustc_middle::ty::Array(..) | rustc_middle::ty::Slice(..)
            );
        if !is_tagged_wrapped {
            return;
        }

        let ast_mutbl = if mutbl.is_mut() {
            rustc_ast::Mutability::Mut
        } else {
            rustc_ast::Mutability::Not
        };
        self.first_pass
            .ref_to_tupleable
            .record(expr.span, self.tcx.sess.source_map(), ast_mutbl);
    }
}
