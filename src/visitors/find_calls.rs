/* This file defines a visitor which is used during the first compiler invocation, to:
 * 1. Find all places where a non-user-defined function was called.
 *    Calls to functions which are not known by self.first_pass are considered
 *    to be untracked function calls, which require special handling later on.
 * 2. Find all places where an array is coereced to a slice
*/

use rustc_hir as hir;
use rustc_hir::def::Res;
use rustc_hir::intravisit::{self, Visitor};
use rustc_middle::hir::nested_filter;
use rustc_middle::ty::{self, TyCtxt};
use rustc_middle::ty::adjustment::{Adjust, PointerCoercion};

use crate::common::CanBeTupled;
use crate::types::ati_info::{
    CompoundTypeInfo, FirstPassInfo, StructFieldInfo, UntrackedReturnKind,
};

/// Visitor that finds all invocations of untracked functions and locations
/// where an array to slice coercion takes place. Updates self.first_pass
/// to include this information after running.
pub struct AnalyzeHirVisitor<'tcx, 'a> {
    pub tcx: TyCtxt<'tcx>,
    pub first_pass: &'a mut FirstPassInfo,
}

impl<'tcx, 'a> AnalyzeHirVisitor<'tcx, 'a> {
    /// Determines the UntrackedReturnKind for a given return type.
    /// For primitives, returns Tupleable. For external structs with all-public
    /// fields, gathers field info and registers a compound type.
    fn classify_untracked_return(&mut self, ret_ty: ty::Ty<'tcx>) -> UntrackedReturnKind {
        // Peel references before checking — untracked functions often return
        // &u32 (e.g. HashMap::get → Option<&V>, then unwrap → &V). The
        // dereference happens implicitly and the value still needs to be tupled.
        let peeled = ret_ty.peel_refs();
        if peeled.can_be_tupled() {
            return UntrackedReturnKind::Tupleable;
        }

        if let ty::TyKind::Adt(adt_def, substs) = ret_ty.kind() {
            // Only handle external (non-local) structs — local structs
            // get their fields tupled in-place by visit_item.
            // Skip generic structs — their type parameters get tupled via
            // recursively_tuple_type on generic args (e.g., Range<u32> → Range<Tagged<u32>>).
            if adt_def.is_struct() && !adt_def.did().is_local() && substs.is_empty() {
                let variant = adt_def.non_enum_variant();

                // Can only convert structs with all-public fields
                let all_public = variant.fields.iter().all(|f| {
                    f.vis.is_public()
                });
                if !all_public || variant.fields.is_empty() {
                    return UntrackedReturnKind::None;
                }

                let struct_name = self.tcx.item_name(adt_def.did()).to_string();
                let tagged_name = format!("__ati_{struct_name}");

                let fields: Vec<StructFieldInfo> = variant
                    .fields
                    .iter()
                    .map(|field| {
                        let field_ty = field.ty(self.tcx, substs);
                        StructFieldInfo {
                            name: field.name.to_string(),
                            ty_str: field_ty.to_string(),
                            should_tuple: field_ty.can_be_tupled(),
                        }
                    })
                    .collect();

                let info = CompoundTypeInfo {
                    original_name: struct_name,
                    tagged_name: tagged_name.clone(),
                    fields,
                };
                self.first_pass.register_compound_type(info);
                return UntrackedReturnKind::Compound(tagged_name);
            }
        }

        UntrackedReturnKind::None
    }
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
            hir::ExprKind::Call(func, _args) => {
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
                            let kind = self.classify_untracked_return(ret_ty);
                            self.first_pass.observe_untracked_fn_call(span, kind);
                        }
                    }
                } else {
                    // TODO: could an instrumented call have a non-path kind?
                    // yes? closures?
                }
            }

            // we are taking a reference to some sort of expression. This is potentially a location
            // where an array to slice coercion is happening.
            hir::ExprKind::AddrOf(..) => {
                let ldid = expr.hir_id.owner.def_id;
                let typeck = self.tcx.typeck(ldid);

                // if it was determine that a type has to become unsized,
                // then a fat pointer is being constructed from some sized type
                let adjustments = typeck.expr_adjustments(expr);
                if adjustments.iter().any(|adjustment| {
                    matches!(adjustment.kind, Adjust::Pointer(PointerCoercion::Unsize))
                }) {
                    self.first_pass.observe_slice_coercion(expr.span);
                }
            }

            // we've found a method call...
            hir::ExprKind::MethodCall(segment, receiver, _args, _fn_span) => {
                let ldid = expr.hir_id.owner.def_id;
                let typeck = self.tcx.typeck(ldid);

                if let Some(def_id) = typeck.type_dependent_def_id(expr.hir_id) {
                    if !self.first_pass.is_fn_def_id_tracked(&def_id) {
                        let recv_ty = typeck.expr_ty(receiver);

                        // Skip methods that Tagged<T> overrides for specific receiver types.
                        // Tagged provides its own len() for arrays and slices that preserves
                        // the tag, so these should not be re-tupled at the boundary.
                        let is_tagged_override = segment.ident.as_str() == "len"
                            && (recv_ty.peel_refs().is_array() || recv_ty.peel_refs().is_slice());

                        if !is_tagged_override {
                            // untracked method call — record it so the second pass
                            // can untuple arguments and tuple the return value.
                            let ret_ty = typeck.expr_ty(expr);
                            let kind = self.classify_untracked_return(ret_ty);
                            self.first_pass.observe_untracked_fn_call(expr.span, kind);
                        }
                    }
                }
            }

            hir::ExprKind::Index(recv, idx, _) => {
                let ldid = expr.hir_id.owner.def_id;
                let typeck = self.tcx.typeck(ldid);
                let idx_ty = typeck.expr_ty(idx);
                if !idx_ty.is_numeric() {
                    self.first_pass.observe_index_by_range(expr.span);
                }

                // Check if the receiver is an untracked container type (e.g. Vec, HashMap).
                // If so, record the span so the second pass can tuple the result.
                let recv_ty = typeck.expr_ty(recv).peel_refs();
                if let ty::TyKind::Adt(adt_def, _) = recv_ty.kind() {
                    if !adt_def.did().is_local()
                        && !recv_ty.is_array()
                        && !recv_ty.is_slice()
                    {
                        let ret_ty = typeck.expr_ty(expr);
                        let kind = self.classify_untracked_return(ret_ty);
                        self.first_pass.observe_untracked_index(expr.span, kind);
                    }
                }
            }
            _ => {}
        }

        intravisit::walk_expr(self, expr);
    }
}
