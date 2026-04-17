/* Defines a visitor which performs all AST-to-AST transformations needed for ATI.
 * This visitor handles both expression instrumentation and type wrapping in a single
 * pass. It does not handle inserting any stub code into the crate, nor renaming any idents.
 *
 * Expression instrumentation (performed in visit_expr):
 * 1. Literals -> tracked, tagged literals (`1` -> `ATI::track(1)`)
 * 2. Arrays -> tracked, tagged arrays (`[1; 3]` -> `ATI::track_array([1; 3])`)
 * 3. Slices -> tracked, tagged slices (`&arr` -> `&ATI::track_slice(&arr)`)
 * 4. If/While conditions are untupled, so they still work.
 * 5. Binary-ops / assign-ops into Block expressions that merge together appropriate tags
 * 6. Indexes in Index expressions are untupled, so the index can be used to access the collection
 *
 * Type wrapping (performed in visit_item / visit_local):
 * - Function params/returns: T -> Tagged<T>
 * - Struct/enum fields: T -> Tagged<T>
 * - Let bindings: T -> Tagged<T>
 * - Turbofish generics: f::<u32> -> f::<Tagged<u32>>
*/
use rustc_ast::mut_visit::{self, MutVisitor};
use rustc_ast::{self as ast, BinOpKind, DUMMY_NODE_ID, GenericArgs, UnOp};
use rustc_ast_pretty::pprust;
use rustc_session::parse::ParseSess;
use rustc_span::{DUMMY_SP, Ident};

use crate::common::{self, CanBeTupled, DatirConfig};
use crate::types::ati_info::FirstPassInfo;

/// Enumerates the different types of operations that can be observed.
enum OpKind {
    Logical,
    Comparison,
    Arithmetic,
}

pub struct TransformVisitor<'a> {
    datir_config: &'a DatirConfig,
    first_pass: &'a FirstPassInfo,
    psess: &'a ParseSess,
}

impl<'a> MutVisitor for TransformVisitor<'a> {
    // define to stop visitor from modifying any expressions used as types
    fn visit_param(&mut self, _node: &mut ast::Param) {}
    // define to stop visitor from modifying anon consts which are required to be of the original type.
    // usually this is for things like array lengths.
    fn visit_anon_const(&mut self, _node: &mut rustc_ast::AnonConst) {}

    /// Updates type annotations on `let` bindings so that primitives like `let x: u32`
    /// become `let x: Tagged<u32>` in sync with the rest of the instrumentation.
    fn visit_local(&mut self, local: &mut ast::Local) {
        if let Some(ty) = &mut local.ty {
            self.recursively_tuple_type(ty);
        }
        mut_visit::walk_local(self, local);
    }

    /// Performs expression instrumentation (literals, binary ops, calls, etc.)
    fn visit_expr(&mut self, expr: &mut ast::Expr) {
        mut_visit::walk_expr(self, expr);

        match &mut expr.kind {
            // Convert all literals into Tagged<T>, if necessary
            ast::ExprKind::Lit(lit) => {
                if lit.can_be_tupled() {
                    *expr = self.tuplify_expr(expr);
                }
            }

            // Assigning to a tagged value should NOT consider the tags as being in the same AT
            // as there is no interaction happening between any values here, just a move into different memory
            ast::ExprKind::Assign(lhs, rhs, _) => {
                // *expr = self.transform_assign(&lhs, &rhs);
            }

            // If this AddrOf operation was found to be a coercion between an array to an unsized slice
            // then convert the Tagged<Array> to a Tagged<Slice>. `&arr[range]` patterns are
            // handled specially so the range's actual bounds (and not just the id) are preserved.
            ast::ExprKind::AddrOf(_, _, _) => {
                if self.first_pass.is_span_ref_type_coercion(&expr.span) {
                    *expr = self.array_to_slice(expr);
                }
            }

            // If this expression constructs an array, create a Tagged<Array> by using the runtime
            // library ATI::track_array call.
            ast::ExprKind::Array(_) | ast::ExprKind::Repeat(_, _) => {
                *expr = self.tuplify_array(expr);
            }

            // Convert all invocations of untracked functions
            // to use the un-tupled values, then bringing the return
            // back into a TaggedValue if necessary.
            ast::ExprKind::Call(func, args) => {
                if let ast::ExprKind::Path(_, path) = &mut func.kind {
                    // Update turbofish generics: f::<u32> -> f::<Tagged<u32>>
                    for segment in path.segments.iter_mut() {
                        self.tuple_generic_args_in_segment(segment);
                    }

                    if let Some(is_tupleable) =
                        self.first_pass.is_untracked_call_ret_tupleable(&func.span)
                    {
                        for arg_expr in args.iter_mut() {
                            // If the arg's resolved type was a reference in the
                            // original (pre-instrumentation) crate, leave it
                            // alone. Untupling `&x` → `(&x).1` would strip the
                            // reference entirely; trait method dispatch and
                            // `Tagged: Deref` already turn `&Tagged<T>` into
                            // `&T` at the callee boundary when needed.
                            if self
                                .first_pass
                                .is_ref_typed_untracked_arg(&arg_expr.span)
                            {
                                continue;
                            }
                            *arg_expr = self.untuple(arg_expr.clone());
                        }

                        // FIXME: again, this is a bit wrong. We are currently ignoring the tracked/untracked boundary,
                        // but you can imagine that an untracked func call returns some struct, which itself contains
                        // values that need to be converted into Tagged<T>s. Right now, that case is entirely ignored,
                        // this works properly if the returned value is a simple primitive.
                        if is_tupleable {
                            *expr = self.tuplify_expr(expr);
                        }
                    }
                }
            }

            // Update turbofish generics on method calls
            // at some point this should include more tracked/untracked boundary logic
            ast::ExprKind::MethodCall(box ast::MethodCall { seg, .. }) => {
                self.tuple_generic_args_in_segment(seg);
            }

            // Transform binary ops to include ATI_ANALYSIS calls to merge tags.
            ast::ExprKind::Binary(op, lhs, rhs) => {
                *expr = self.transform_binary_op(&lhs, op.node, &rhs);
            }

            // Transform similar to function transformation, tagging input / output types
            ast::ExprKind::Closure(box ast::Closure {
                fn_decl,
                ..
            }) => {
                // fn_decl.inputs.clone()
                for input in fn_decl.inputs.iter_mut() {
                    self.recursively_tuple_type(&mut input.ty)
                }

                if let ast::FnRetTy::Ty(ty) = &mut fn_decl.output {
                    self.recursively_tuple_type(ty);
                }
            }

            // After Binary transformation, comparison conditions produce Tagged<bool>.
            // Unwrap to raw bool so the if/while condition compiles.
            ast::ExprKind::If(cond, _, _) => {
                *cond = self.untuple(cond.clone());
            },

            ast::ExprKind::While(cond, _, _) => {
                *cond = self.untuple(cond.clone());
            }

            ast::ExprKind::Index(receiver_expr, index_expr, _) => {
                if self.first_pass.is_span_index_by_range(&expr.span) {
                    // index_expr is some range, how should we treat ranges???

                } else {
                    // TODO: change this to instead utilize std::ops::Index trait defined on Tagged<T>s? 
                    // specifically this needs to have the index interact with the length tag.
                    // same thing needs to happpen for ranges
                    // *receiver_expr = self.untuple(receiver_expr.clone());
                    // *index_expr = self.untuple(index_expr.clone());
                }
            }

            // Transform range construction into a tracked-range constructor call.
            // By this point walk_expr has already instrumented the endpoints (so
            // literals/vars are Tagged<T>), and we hand them to the constructor
            // which unions the wrapper id with each endpoint — making the range
            // an AT that contains its endpoints, any value yielded during
            // iteration, `len()`, and any slice produced through indexing.
            ast::ExprKind::Range(lo, hi, limits) => {
                let is_inclusive = matches!(limits, ast::RangeLimits::Closed);
                let code = match (lo.as_ref(), hi.as_ref(), is_inclusive) {
                    (Some(lo), Some(hi), false) => format!(
                        "ATI::track_range({}, {})",
                        pprust::expr_to_string(lo),
                        pprust::expr_to_string(hi),
                    ),
                    (Some(lo), Some(hi), true) => format!(
                        "ATI::track_range_inclusive({}, {})",
                        pprust::expr_to_string(lo),
                        pprust::expr_to_string(hi),
                    ),
                    (Some(lo), None, _) => format!(
                        "ATI::track_range_from({})",
                        pprust::expr_to_string(lo),
                    ),
                    (None, Some(hi), false) => format!(
                        "ATI::track_range_to({})",
                        pprust::expr_to_string(hi),
                    ),
                    (None, Some(hi), true) => format!(
                        "ATI::track_range_to_inclusive({})",
                        pprust::expr_to_string(hi),
                    ),
                    (None, None, _) => "ATI::track_range_full()".to_string(),
                };
                *expr = common::parse_expr(self.psess, code);
            }

            _ => {}
        }
    }

    /// Wraps types in function signatures, struct definitions, and enum definitions.
    /// For functions, walks the body first (triggering visit_expr for expression
    /// instrumentation), then modifies the signature types.
    fn visit_item(&mut self, item: &mut ast::Item) {
        match &mut item.kind {
            ast::ItemKind::Fn(box ast::Fn {
                ident,
                sig: ast::FnSig { decl, .. },
                body,
                ..
            }) => {
                if !self.first_pass.is_fn_ident_tracked(ident) {
                    return;
                }

                // instrument the actual code
                if let Some(body) = body {
                    mut_visit::walk_block(self, body);
                }

                // wrap parameter types in Tagged<T>, as necessary
                for param in &mut decl.inputs {
                    self.recursively_tuple_type(&mut param.ty);
                }

                // wrap return type in Tagged<T>
                if let ast::FnRetTy::Ty(return_type) = &mut decl.output {
                    self.recursively_tuple_type(return_type);
                }
            }

            ast::ItemKind::Struct(_ident, _generics, ast::VariantData::Struct { fields, .. }) => {
                for field_def in fields.iter_mut() {
                    self.recursively_tuple_type(&mut field_def.ty);
                }
            }

            ast::ItemKind::Enum(_ident, _, ast::EnumDef { variants }) => {
                for variant in variants.iter_mut() {
                    match &mut variant.data {
                        ast::VariantData::Struct { fields, .. } => {
                            for field in fields.iter_mut() {
                                self.recursively_tuple_type(&mut field.ty);
                            }
                        }
                        ast::VariantData::Tuple(fields, _) => {
                            for field in fields.iter_mut() {
                                self.recursively_tuple_type(&mut field.ty);
                            }
                        }
                        ast::VariantData::Unit(_) => {}
                    }
                }
            }

            ast::ItemKind::Impl(ast::Impl {
                items,
                ..
            }) => {
                for assoc_item in items.iter_mut() {
                    let ast::AssocItemKind::Fn(box ast::Fn {
                        ident,
                        sig: ast::FnSig { decl, .. },
                        body,
                        ..
                    }) = &mut assoc_item.kind
                    else {
                        continue;
                    };

                    if !self.first_pass.is_fn_ident_tracked(ident) {
                        continue;
                    }

                    // instrument method body
                    if let Some(body) = body {
                        mut_visit::walk_block(self, body);
                    }

                    // tag all non-self parameter types
                    for param in &mut decl.inputs {
                        if !Self::is_self_param(param) {
                            self.recursively_tuple_type(&mut param.ty);
                        }
                    }

                    // wrap return type
                    if let ast::FnRetTy::Ty(ret_ty) = &mut decl.output {
                        self.recursively_tuple_type(ret_ty);
                    }
                }
            }

            _ => {}
        }
    }
}

impl<'a> TransformVisitor<'a> {
    /// Consutrctor
    pub fn new(datir_config: &'a DatirConfig, first_pass: &'a FirstPassInfo, psess: &'a ParseSess) -> Self {
        Self { datir_config, first_pass, psess }
    }

    ///////////////// Expression Instrumentation Helpers //////////////////////
    
    /// Transforms lhs = rhs into a block which merges together the tags of the lhs and rhs expression
    /// and then does the actual assignment.
    fn transform_assign(&self, lhs: &ast::Expr, rhs: &ast::Expr) -> ast::Expr {
        let lhs_str = pprust::expr_to_string(lhs);
        let rhs_str = pprust::expr_to_string(rhs);

        let assign_expr = format!("{lhs_str} = {{
            let __ati_lhs = {lhs_str};
            let __ati_rhs = {rhs_str};
            ATI_ANALYSIS.lock().unwrap().union_and_get_id(&__ati_lhs.0, &__ati_rhs.0);
            __ati_rhs
        }}");

        common::parse_expr(self.psess, assign_expr)
    }

    /// Transforms `lhs op rhs` (where we expect both operands are Tagged<T>s) into a block that
    /// explicitly calls ATI_ANALYSIS functions to record the interaction and constructs the result.
    fn transform_binary_op(&self, lhs: &ast::Expr, op: BinOpKind, rhs: &ast::Expr) -> ast::Expr {
        let lhs_str = pprust::expr_to_string(lhs);
        let rhs_str = pprust::expr_to_string(rhs);
        let op_str = op.as_str();

        // FIXME: Kind of stupid to go from lhs op rhs to lhs op rhs in arithemtic case
        let block_str = match Self::op_type(op) {
            OpKind::Comparison => {
                // guaranteed that lhs op rhs will return a regular bool
                format!(
                    r#"{{
                        let __ati_id = ATI_ANALYSIS.lock().unwrap().make_id();
                        Tagged(__ati_id, {lhs_str} {op_str} {rhs_str})
                    }}"#
                )
            }
            OpKind::Logical => {
                // guaranteed that lhs and rhs were bools, and are now Tagged<bool>s
                format!(
                    r#"{{
                        let __ati_lhs = {lhs_str};
                        let __ati_rhs = {rhs_str};
                        let __ati_id = ATI_ANALYSIS.lock().unwrap().union_and_get_id(&__ati_lhs.0, &__ati_rhs.0);
                        Tagged(__ati_id, __ati_lhs.1 {op_str} __ati_rhs.1)
                    }}"#
                )
            }
            OpKind::Arithmetic => {
                // handled by op impls on Tagged<T>
                format!(
                    r#"{{
                        ({lhs_str} {op_str} {rhs_str})
                    }}"#
                )
            }
        };

        // self.datir_config.log("TMP", format!("======\n{block_str}"));

        common::parse_expr(self.psess, block_str)
    }

    /// Converts between rustc's BinOpKind type to DATIR's OpKind type
    fn op_type(op: BinOpKind) -> OpKind {
        match op {
            BinOpKind::Eq
            | BinOpKind::Ne
            | BinOpKind::Lt
            | BinOpKind::Gt
            | BinOpKind::Le
            | BinOpKind::Ge => OpKind::Comparison,

            | BinOpKind::BitXor
            | BinOpKind::BitAnd
            | BinOpKind::BitOr
            | BinOpKind::Shl
            | BinOpKind::Shr
            | BinOpKind::Add
            | BinOpKind::Sub
            | BinOpKind::Mul
            | BinOpKind::Div
            | BinOpKind::Rem => OpKind::Arithmetic,

            BinOpKind::And
            | BinOpKind::Or => OpKind::Logical,

        }
    }

    /// Rewrites a slice-coercion `AddrOf` expression into the matching ATI helper call:
    ///   - `&arr`            -> `ATI::track_slice(&arr)`
    ///   - `&mut arr`        -> `ATI::track_slice_mut(&mut arr)`
    ///   - `&arr[range]`     -> `ATI::track_subslice(&arr, range)`
    ///   - `&mut arr[range]` -> `ATI::track_subslice_mut(&mut arr, range)`
    /// The source-level outer `&`/`&mut` is absorbed into the helper's return type
    /// (`Tagged<&[T]>` / `Tagged<&mut [T]>`), matching the `tuple_slice` type-wrapping
    /// convention. In the range-index case the original `range` expression is preserved
    /// so the produced slice still covers the caller-specified bounds at runtime. Range
    /// endpoints have already been untupled by the `Range` arm in `visit_expr` before
    /// this runs, so `SliceIndex` sees raw `usize` bounds.
    fn array_to_slice(&self, expr: &mut ast::Expr) -> ast::Expr {
        let ast::ExprKind::AddrOf(_, mutbl, inner) = &expr.kind else {
            unimplemented!(
                "Only reference-based fat pointers are supported for array -> slice coercion"
            );
        };

        let amp = match mutbl {
            ast::Mutability::Mut => "&mut ",
            ast::Mutability::Not => "&",
        };
        let is_mut = matches!(mutbl, ast::Mutability::Mut);

        let code = if let ast::ExprKind::Index(recv, idx, _) = &inner.kind {
            let recv_str = pprust::expr_to_string(recv);
            let idx_str = pprust::expr_to_string(idx);
            let fn_name = if is_mut { "track_subslice_mut" } else { "track_subslice" };
            format!("{amp}ATI::{fn_name}({recv_str}, {idx_str})")
        } else {
            let inner_str = pprust::expr_to_string(inner);
            let fn_name = if is_mut { "track_slice_mut" } else { "track_slice" };
            format!("{amp}ATI::{fn_name}({amp}{inner_str})")
        };

        common::parse_expr(self.psess, code)
    }

    /// Converts from a [T; N] to a Tagged<[T; N]>
    fn tuplify_array(&self, expr: &mut ast::Expr) -> ast::Expr {
        let mut receiver_expr = ast::Expr::dummy();
        receiver_expr.kind = ast::ExprKind::Path(
            None,
            ast::Path {
                span: DUMMY_SP,
                segments: [
                    ast::PathSegment {
                        ident: Ident::from_str("ATI"),
                        id: DUMMY_NODE_ID,
                        args: None,
                    },
                    ast::PathSegment {
                        ident: Ident::from_str("track_array"),
                        id: DUMMY_NODE_ID,
                        args: None,
                    },
                ]
                .into(),
                tokens: None,
            },
        );

        let mut new_expr = ast::Expr::dummy();
        new_expr.kind =
            ast::ExprKind::Call(Box::new(receiver_expr), [Box::new(expr.clone())].into());

        new_expr
    }

    /// Takes an expression of type T and converts it to an expression of Tagged<T>
    fn tuplify_expr(&self, expr: &ast::Expr) -> ast::Expr {
        ast::Expr {
            id: ast::DUMMY_NODE_ID,
            kind: ast::ExprKind::Call(
                Box::new(ast::Expr {
                    id: ast::DUMMY_NODE_ID,
                    kind: ast::ExprKind::Path(
                        None,
                        ast::Path {
                            span: DUMMY_SP,
                            segments: [
                                ast::PathSegment {
                                    ident: Ident::from_str("ATI"),
                                    id: ast::DUMMY_NODE_ID,
                                    args: None,
                                },
                                ast::PathSegment {
                                    ident: Ident::from_str("track"),
                                    id: ast::DUMMY_NODE_ID,
                                    args: None,
                                },
                            ]
                            .into(),
                            tokens: None,
                        },
                    ),
                    span: DUMMY_SP,
                    attrs: [].into(),
                    tokens: None,
                }),
                [Box::new(expr.clone())].into(),
            ),
            span: DUMMY_SP,
            attrs: [].into(),
            tokens: None,
        }
    }

    /// Takes a Tagged<T> expression and unwraps it to just T
    fn untuple(&self, expr: Box<ast::Expr>) -> Box<ast::Expr> {
        let mut node = ast::Expr::dummy();
        node.kind = ast::ExprKind::Field(expr, Ident::from_str("1"));
        Box::new(node)
    }

    ///////////////// Type Wrapping Helpers //////////////////////

    /// Directly modifies a type T into a Tagged<T> in place
    fn tuple_type(&self, old_type: &mut ast::Ty) {
        old_type.kind = ast::TyKind::Path(
            None,
            ast::Path {
                segments: [ast::PathSegment {
                    ident: Ident::from_str("Tagged"),
                    id: ast::DUMMY_NODE_ID,
                    args: Some(Box::new(ast::AngleBracketed(ast::AngleBracketedArgs {
                        span: DUMMY_SP,
                        args: [ast::AngleBracketedArg::Arg(ast::GenericArg::Type(
                            Box::new(old_type.clone()),
                        ))]
                        .into(),
                    }))),
                }]
                .into(),
                span: DUMMY_SP,
                tokens: None,
            },
        );
    }

    /// Converts a `&(mut)? [T]` into `Tagged<&(mut)? [T]>`. The source-level
    /// outer reference is absorbed into the `Tagged` wrapper, so slice types
    /// follow the same "value-shaped Tagged wrapper" convention as arrays
    /// (`[T; N]` -> `Tagged<[Tagged<T>; N]>`). Matching `array_to_slice` below
    /// produces no outer `&`, which keeps array literals of slice references
    /// from borrowing short-lived temporaries.
    fn tuple_slice(&self, outer_ty: &mut ast::Ty) {
        let ast::TyKind::Ref(lt, ast::MutTy {
            ty,
            mutbl,
        }) = &mut outer_ty.kind else {
            panic!("Non-reference based slice type is unsupported");
        };


        let mut tagged_slice = ast::PathSegment::from_ident(Ident::from_str("Tagged"));
        tagged_slice.args = Some(Box::new(GenericArgs::AngleBracketed(
            ast::AngleBracketedArgs {
                span: DUMMY_SP,
                args: [ast::AngleBracketedArg::Arg(ast::GenericArg::Type(
                    Box::new(
                        ast::Ty {
                            id: DUMMY_NODE_ID,
                            kind: ast::TyKind::Ref(lt.clone(), ast::MutTy {
                                ty: ty.clone(),
                                mutbl: mutbl.clone()
                            }),
                            span: DUMMY_SP,
                            tokens: None,
                        },
                    ),
                ))]
                .into(),
            },
        )));

        ty.kind = ast::TyKind::Path(
            None,
            ast::Path {
                span: DUMMY_SP,
                segments: [tagged_slice].into(),
                tokens: None,
            },
        );
    }

    /// Converts a [T; N] into a Tagged<[T; N]>
    fn tuple_array(&self, array_ty: &mut ast::Ty) {
        let mut tagged_array = ast::PathSegment::from_ident(Ident::from_str("Tagged"));
        tagged_array.args = Some(Box::new(GenericArgs::AngleBracketed(
            ast::AngleBracketedArgs {
                span: DUMMY_SP,
                args: [ast::AngleBracketedArg::Arg(ast::GenericArg::Type(
                    Box::new(array_ty.clone()),
                ))]
                .into(),
            },
        )));

        array_ty.kind = ast::TyKind::Path(
            None,
            ast::Path {
                span: DUMMY_SP,
                segments: [tagged_array].into(),
                tokens: None,
            },
        );
    }

    /// Recursively tuples all type generic arguments in a path segment
    fn tuple_generic_args_in_segment(&self, segment: &mut ast::PathSegment) {
        let Some(ref mut boxed_args) = segment.args else {
            return;
        };
        let ast::GenericArgs::AngleBracketed(ast::AngleBracketedArgs { ref mut args, .. }) =
            **boxed_args
        else {
            return;
        };
        for arg in args.iter_mut() {
            if let ast::AngleBracketedArg::Arg(ast::GenericArg::Type(ty)) = arg {
                self.recursively_tuple_type(ty);
            }
        }
    }

    /// returns true if param is a self receiver (`self`, `&self`, `&mut self`)
    fn is_self_param(param: &ast::Param) -> bool {
        matches!(param.ty.peel_refs().kind, ast::TyKind::ImplicitSelf)
    }

    /// Searches through type `ty` to find and tuple all primitive types
    /// that should be tupled. Modifies the type in place.
    fn recursively_tuple_type(&self, ty: &mut ast::Ty) {
        let peeled_type = common::peel_refs(ty);

        // base case, the type can just be tupled and no recursion is necessary
        if peeled_type.can_be_tupled() {
            self.tuple_type(peeled_type);
            return;
        }

        match &mut peeled_type.kind {
            rustc_ast::TyKind::Slice(inner_ty) => {
                self.recursively_tuple_type(inner_ty);
                self.tuple_slice(ty);
            }

            rustc_ast::TyKind::Array(inner_ty, _) => {
                self.recursively_tuple_type(inner_ty);
                self.tuple_array(peeled_type);
            }

            rustc_ast::TyKind::Ptr(ast::MutTy { box ty, .. })
            | rustc_ast::TyKind::Ref(_, ast::MutTy { box ty, .. }) => {
                self.recursively_tuple_type(ty);
            }

            rustc_ast::TyKind::FnPtr(box ast::FnPtrTy {
                generic_params,
                decl: box ast::FnDecl { inputs, output },
                ..
            }) => {
                for generic in generic_params {
                    match &mut generic.kind {
                        rustc_ast::GenericParamKind::Const { ty, default, .. } => {
                            self.recursively_tuple_type(ty);
                            // FIXME: handle the default value. An AnonConst isn't computed
                            // at runtime though, so how are we associating an Id with it?
                        }
                        _ => {}
                        // Pretty certain we want to leave generics alone
                        // rustc_ast::GenericParamKind::Type { default } => {
                        //     if let Some(ty) = default {
                        //         self.recursively_tuple_type(ty);
                        //     }
                        // }
                        // rustc_ast::GenericParamKind::Lifetime => {}
                    }
                }

                for input in inputs {
                    self.recursively_tuple_type(&mut input.ty)
                }

                if let ast::FnRetTy::Ty(box ty) = output {
                    self.recursively_tuple_type(ty);
                }
            }

            rustc_ast::TyKind::Tup(tys) => {
                for ty in tys {
                    self.recursively_tuple_type(ty);
                }
            }

            rustc_ast::TyKind::Path(_, ast::Path { segments, .. }) => {
                for segment in segments.iter_mut() {
                    if let Some(box arg) = &mut segment.args {
                        match arg {
                            rustc_ast::GenericArgs::AngleBracketed(ast::AngleBracketedArgs {
                                args,
                                ..
                            }) => {
                                for arg in args.iter_mut() {
                                    match arg {
                                        rustc_ast::AngleBracketedArg::Arg(generic_arg) => {
                                            match generic_arg {
                                                rustc_ast::GenericArg::Type(ty) => {
                                                    self.recursively_tuple_type(ty);
                                                }
                                                rustc_ast::GenericArg::Const(_)
                                                | rustc_ast::GenericArg::Lifetime(_) => {}
                                            }
                                        }
                                        rustc_ast::AngleBracketedArg::Constraint(_) => {
                                            todo!("Constraint is a trait?")
                                        }
                                    }
                                }
                            }
                            rustc_ast::GenericArgs::Parenthesized(ast::ParenthesizedArgs {
                                inputs,
                                output,
                                ..
                            }) => {
                                for input in inputs {
                                    self.recursively_tuple_type(input);
                                }

                                if let ast::FnRetTy::Ty(box ty) = output {
                                    self.recursively_tuple_type(ty);
                                }
                            }
                            rustc_ast::GenericArgs::ParenthesizedElided(_span) => {
                                panic!("this panic is probably fine to remove")
                            }
                        }
                    }
                }

                // FIXME: this is not resilient to custom types that are called "range".
                // If this path refers to a std range type, wrap the whole
                // type in Tagged so `std::ops::Range<usize>` becomes
                // `Tagged<std::ops::Range<Tagged<usize>>>`.
                if let Some(last) = segments.last() {
                    let name = last.ident.name.as_str();
                    if matches!(
                        name,
                        "Range"
                            | "RangeInclusive"
                            | "RangeFrom"
                            | "RangeTo"
                            | "RangeToInclusive"
                            | "RangeFull"
                    ) {
                        self.tuple_type(peeled_type);
                    }
                }
            }

            // maybe impl later
            rustc_ast::TyKind::PinnedRef(_, _) => todo!(),
            rustc_ast::TyKind::Pat(_, _) => todo!(),

            // probably left untouched
            rustc_ast::TyKind::Infer => {},
            rustc_ast::TyKind::TraitObject(_, _) => panic!(),
            rustc_ast::TyKind::Paren(_) => panic!(),
            rustc_ast::TyKind::UnsafeBinder(_) => panic!(),
            rustc_ast::TyKind::Never => panic!(),
            rustc_ast::TyKind::ImplTrait(_, _) => panic!(),
            rustc_ast::TyKind::ImplicitSelf => panic!(),
            rustc_ast::TyKind::MacCall(_) => panic!(),
            rustc_ast::TyKind::CVarArgs => panic!(),
            rustc_ast::TyKind::Dummy => panic!(),
            rustc_ast::TyKind::Err(_) => panic!(),
        };
    }
}
