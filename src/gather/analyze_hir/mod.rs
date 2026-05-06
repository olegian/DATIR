/* This file defines a visitor which is used during the first compiler invocation, to:
 * 1. Find all places where a non-user-defined function was called.
 *    Calls to functions which are not known by self.first_pass are considered
 *    to be untracked function calls, which require special handling later on.
 * 2. Find all places where a reference is constructed to some tuplable type.
 * 3. Find all places where a range is used as an index into some collection.
 * 4. Find all places where a mutable reference is assigned to.
 * 5. Find all places where a unary deref operation is used on a TaggedRef(Mut?) and the
 *    result needs to net a Tagged<T> as opposed to a T which is offered by the standard
 *    deref implementation.
*/

use rustc_hir as hir;
use rustc_hir::intravisit::{self};

use crate::{common::CanBeTupled, gather::first_pass::FirstPassInfo};

mod assignment;
mod call;
mod deref;
mod index;
mod references;

/// Visitor that finds code spans of interest (listed at the top of this file).
/// Updates self.first_pass to include this information after running.
pub struct AnalyzeHirVisitor<'tcx, 'a> {
    pub tcx: rustc_middle::ty::TyCtxt<'tcx>,
    pub first_pass: &'a mut FirstPassInfo,
}

impl<'tcx, 'a> rustc_hir::intravisit::Visitor<'tcx> for AnalyzeHirVisitor<'tcx, 'a> {
    type NestedFilter = rustc_middle::hir::nested_filter::All;

    /// Combined with above NestedFilter, defines how the visitor
    /// is going to traverse the tree. This configuration will have
    /// this visitor visit all nested expressions, as in we are doing
    /// a "deep" traversal, visiting every single expression as opposed
    /// to doing a "shallow" traversal, visiting only the top-level exprs
    fn maybe_tcx(&mut self) -> Self::MaybeTyCtxt {
        self.tcx
    }

    /// Anon consts (array lengths, const generics, inline consts) live in
    /// their own owner with no typeck results, and have no values for us to
    /// instrument. Skip the entire subtree.
    fn visit_anon_const(&mut self, _: &'tcx hir::AnonConst) {}

    /// Called on each expression.
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        // Skip subtrees whose owner has no typeck results (e.g. struct/enum
        // item bodies reached via nested-filter walks, inline consts).
        let ldid = expr.hir_id.owner.def_id;
        if !self.tcx.has_typeck_results(ldid) {
            return;
        }
        // Kind-independent: record any expression whose adjusted type is
        // `&mut T` with `T` tupleable. Pass 2 must reborrow such operands
        // before consuming them; see `FirstPassInfo::ref_mut_to_tupleable_locs`.
        // Use the *declared* type (not adjusted): pass 2 reborrows the
        // operand right before moving it into a synthesized binding (e.g.
        // `let __ati_lhs = <expr>;` in the binary-op rewrite). Auto-ref /
        // auto-deref happens at the consumer of that binding and doesn't
        // change whether the move itself sees a `&mut T`. For example, in
        // `a == b` with `a: &mut u32`, `expr_ty_adjusted` reports `&&mut
        // u32` (the auto-ref for PartialEq::eq), but the move into the
        // synthesized let still consumes a bare `&mut u32`.
        let typeck = self.tcx.typeck(ldid);
        let expr_ty = typeck.expr_ty(expr);
        if let rustc_middle::ty::Ref(_, referent, mutbl) = *expr_ty.kind() {
            if mutbl.is_mut() && referent.can_be_tupled() {
                self.first_pass
                    .observe_ref_mut_to_tupleable(expr.span, self.tcx.sess.source_map());
            }
        }

        match expr.kind {
            // A call to a function might require us to untuple the arguments,
            // and then tuple back the return value, if it is a call to a function
            // which we are not going to be instrumenting.
            hir::ExprKind::Call(..) => {
                self.observe_call(expr);
            }

            // we are taking a reference to some sort of expression. If the
            // reference is to some type which is tuplable (e.g. &u32, or &mut &f64)
            // then during instrumentation we need to create a TaggedRef<T> from the Tagged<T>.
            hir::ExprKind::AddrOf(..) => {
                self.observe_ref(expr);
            }

            // Unary * on an instrumented &T / &mut T with tupleable T
            // strips the tag post-instrumentation (TaggedRef::deref -> T). Record
            // the span so pass 2 can rebuild a Tagged<T> from the borrowed fields,
            // and as a result have *&TaggedRef<T> net a Tagged<T>.
            // FIXME: In the future, this has to be fixed up to allow for all smart pointers.
            // For now, smart pointers (Box) stay using a plain `*`.
            hir::ExprKind::Unary(hir::UnOp::Deref, _) => {
                self.observe_deref(expr);
            }

            // Assignment (or compound assign) whose LHS is *expr where expr
            // is &mut T with tupleable T. Post-instrumentation the LHS is a
            // TaggedRefMut<T>; a plain *lhs = rhs goes through DerefMut and
            // only touches the value field (.1), leaving the old id (.0) behind.
            // Record the span so pass 2 rewrites it to expr.assign(rhs), which writes both fields.
            // .assign is defined in the runtime library, on the TaggedRefMut type.
            hir::ExprKind::Assign(..) | hir::ExprKind::AssignOp(..) => {
                self.observe_assignment(expr);
            }

            // Indexing is usually handled via traits defined on the Tagged* types in the
            // runtime library. Ranges are special cased however, and SliceIndex cannot be
            // overloaded in the way that the Index operation can. Therefore, we have to
            // record places where a range is used as an index, to correctly transform it
            // to the appropriate subslice operation in the next pass.
            hir::ExprKind::Index(..) => {
                self.observe_range(expr);
            }

            _ => {}
        }

        intravisit::walk_expr(self, expr);
    }
}
