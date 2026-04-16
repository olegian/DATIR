/* This file defines a visitor which is used during the first compiler invocation, to:
 * 1. Find all places where a non-user-defined function was called.
 *    Calls to functions which are not known by self.first_pass are considered
 *    to be untracked function calls, which require special handling later on.
 * 2. Find all places where an array is coereced to a slice
*/

use rustc_hir as hir;
use rustc_hir::def::Res;
use rustc_hir::def_id::DefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_middle::hir::nested_filter;
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty::adjustment::{Adjust, PointerCoercion};

use crate::types::ati_info::FirstPassInfo;

/// Lang items for every range struct that is valid as an indexing argument and
/// produces a slice result (`Range`, `RangeFrom`, `RangeFull`, `RangeInclusive`,
/// `RangeTo`, `RangeToInclusive`, and their `*Copy` variants).
const RANGE_LANG_ITEMS: &[rustc_hir::LangItem] = &[
    rustc_hir::LangItem::Range,
    rustc_hir::LangItem::RangeFrom,
    rustc_hir::LangItem::RangeFull,
    rustc_hir::LangItem::RangeInclusiveStruct,
    rustc_hir::LangItem::RangeTo,
    rustc_hir::LangItem::RangeToInclusive,
    rustc_hir::LangItem::RangeCopy,
    rustc_hir::LangItem::RangeFromCopy,
    rustc_hir::LangItem::RangeInclusiveCopy,
    rustc_hir::LangItem::RangeToInclusiveCopy,
];

fn is_range_lang_item<'tcx>(tcx: TyCtxt<'tcx>, did: DefId) -> bool {
    RANGE_LANG_ITEMS
        .iter()
        .any(|&lang| tcx.is_lang_item(did, lang))
}

/// Visitor that finds all invocations of untracked functions and locations
/// where an array to slice coercion takes place. Updates self.first_pass
/// to include this information after running.
pub struct AnalyzeHirVisitor<'tcx, 'a> {
    pub tcx: TyCtxt<'tcx>,
    pub first_pass: &'a mut FirstPassInfo,
}

impl<'tcx, 'a> Visitor<'tcx> for AnalyzeHirVisitor<'tcx, 'a> {
    type NestedFilter = nested_filter::All;

    /// Combined with above NestedFilter, defines how the visitor
    /// is going to traverse the tree. This configuration will have
    /// this visitor visit all nested expressions, as in we are doing
    /// a "deep" traversal, visiting every single expression as opposed
    /// to doing a "shallow" traversal, visiting only the top-level exprs
    fn maybe_tcx(&mut self) -> Self::MaybeTyCtxt {
        self.tcx
    }

    /// Called on each expression.
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        match expr.kind {
            // we've found a call to a function...
            hir::ExprKind::Call(func, args) => {
                if let hir::ExprKind::Path(ref qpath) = func.kind {
                    let ldid = expr.hir_id.owner.def_id;

                    let typeck = self.tcx.typeck(ldid);
                    if let Res::Def(kind, def_id) = typeck.qpath_res(qpath, func.hir_id) {
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
                            self.first_pass.observe_untracked_fn_call(span, ret_ty);

                            // Per-arg: any arg whose resolved type is a
                            // reference must not be untupled. Stripping `.1`
                            // would discard the reference (auto-deref field
                            // access copies/moves the inner value), whereas
                            // method/fn dispatch on a `&Tagged<T>` auto-derefs
                            // into `&T` at the callee boundary already.
                            for arg in args.iter() {
                                let arg_ty = typeck.expr_ty(arg);
                                if arg_ty.is_ref() {
                                    self.first_pass.observe_ref_typed_untracked_arg(arg.span);
                                }
                            }
                        }
                    }
                } else {
                    // TODO: could an instrumented call have a non-path kind?
                    // yes? closures?
                }
            }

            // we are taking a reference to some sort of expression. This is potentially a location
            // where an array to slice coercion is happening.
            hir::ExprKind::AddrOf(_, _, inner) => {
                let ldid = expr.hir_id.owner.def_id;
                let typeck = self.tcx.typeck(ldid);

                // if it was determine that a type has to become unsized,
                // then a fat pointer is being constructed from some sized type
                let adjustments = typeck.expr_adjustments(expr);
                let has_unsize = adjustments.iter().any(|adjustment| {
                    matches!(adjustment.kind, Adjust::Pointer(PointerCoercion::Unsize))
                });

                // `&arr[range]` / `&mut arr[range]` produces a slice reference directly,
                // without an Unsize coercion. Detect any range-typed indexing here so
                // all range variants (`..`, `1..3`, `..2`, `1..`, `1..=3`, `..=3`) are
                // treated as slice coercion sites.
                let has_range_index = if let hir::ExprKind::Index(_, idx, _) = inner.kind {
                    typeck
                        .expr_ty(idx)
                        .ty_adt_def()
                        .map(|adt| is_range_lang_item(self.tcx, adt.did()))
                        .unwrap_or(false)
                } else {
                    false
                };

                if has_unsize || has_range_index {
                    self.first_pass.observe_slice_coercion(expr.span);
                }
            }

            hir::ExprKind::Index(recv, idx, _) => {
                let ldid = expr.hir_id.owner.def_id;
                let typeck = self.tcx.typeck(ldid);
                let idx_ty = typeck.expr_ty(idx);
                if !idx_ty.is_numeric() {
                    self.first_pass.observe_index_by_range(expr.span);
                }
            }
            _ => {}
        }

        intravisit::walk_expr(self, expr);
    }
}
