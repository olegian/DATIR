/* Before we can perform the required AST mutation, we need to gather
 * some type information about the original source code. This is done by
 * invoking the compiler and passing in the GatherAtiInfo callback struct
 * defined in this file. See after_expansion below for more specific information
 * on what information is gathered.
*/
use rustc_ast as ast;
use rustc_driver::Compilation;
use rustc_hir::def_id::{CRATE_DEF_ID, LocalDefId};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;
use std::sync::Arc;

use decls_gen::{DeclsFile, VarIdent};

use crate::{
    common::DatirConfig,
    types::ati_info::{FirstPassInfo, ModPath, TypeKey},
    visitors::AnalyzeHirVisitor,
};

/// Defines the callbacks used for the first information-gathering compilation.
pub struct GatherAtiInfo {
    /// Contains the information discovered after executing the compilation.
    first_pass: FirstPassInfo,
    config: Arc<DatirConfig>,
}

impl GatherAtiInfo {
    /// Constructor
    pub fn new(config: Arc<DatirConfig>) -> Self {
        Self {
            first_pass: Default::default(),
            config,
        }
    }

    /// Pulls out all gathered info that this compiler invocation learned.
    /// Must be called after the first compilation is performed.
    pub fn into_first_pass_info(self) -> FirstPassInfo {
        self.first_pass
    }

    /// For the given function identified by `local_def_id`, get the base_ppt_name
    /// which corresponds to it (i.e. everything before :::{ENTER|EXIT|EXITNN} in
    /// the decls file). Validate that the loaded decls file contains the matching
    /// ENTER and EXIT program points, that every formal parameter has a
    /// `VariableDecl` on both, and that any non-unit return value has a return
    /// `VariableDecl` on EXIT. Store the fn ident, def id, and base_ppt_name in
    /// FirstPassInfo.
    fn record_fn<'tcx>(
        &mut self,
        tcx: TyCtxt<'tcx>,
        local_def_id: LocalDefId,
        ident: rustc_span::Ident,
        type_key: Option<TypeKey>,
    ) {
        let base_ppt_name = DeclsFile::ppt_base_name(tcx, local_def_id);
        let decls_file = &self.config.decls_file;

        // make sure the decls file has an appropriate enter and exit ppt 
        // defined for this base_ppt_name. Otherwise, the instrumented 
        // binary is going to emit comparability information that is impossible
        // to associate with any ppt.
        let enter_ppt = decls_file.enter_ppt(&base_ppt_name).unwrap_or_else(|| {
            panic!(
                "DATIR/decls-gen is out of sync: no ENTER program point in the .decls \
                 file matches base ppt name `{base_ppt_name}` for {local_def_id:?}."
            )
        });
        let exit_ppt = decls_file.exit_ppt(&base_ppt_name).unwrap_or_else(|| {
            panic!(
                "DATIR/decls-gen is out of sync: no EXIT program point in the .decls \
                 file matches base ppt name `{base_ppt_name}` for {local_def_id:?}."
            )
        });

        // Make sure that all formals/return values are properly included in the DeclsFile too,
        // at least by top-level name. 
        // FIXME: is it worth it to recrusively descend here and check children?
        let body = tcx.hir_body_owned_by(local_def_id);
        for param in body.params.iter() {
            let formal = param
                .pat
                .simple_ident()
                .unwrap_or_else(|| {
                    panic!(
                        "Formal parameter of `{base_ppt_name}` is not a simple ident pattern."
                    )
                })
                .name
                .to_string();

            if enter_ppt
                .var_decl_lookup(tcx, VarIdent::Local(formal.clone()))
                .is_none()
            {
                panic!(
                    "DATIR/decls-gen is out of sync: ENTER ppt `{base_ppt_name}:::ENTER` \
                     is missing a VariableDecl for formal `{formal}`."
                );
            }
            if exit_ppt
                .var_decl_lookup(tcx, VarIdent::Local(formal.clone()))
                .is_none()
            {
                panic!(
                    "DATIR/decls-gen is out of sync: EXIT ppt `{base_ppt_name}:::EXIT` \
                     is missing a VariableDecl for formal `{formal}`."
                );
            }
        }

        let return_ty = tcx
            .fn_sig(local_def_id)
            .instantiate_identity()
            .skip_binder()
            .output();
        if !return_ty.is_unit() && exit_ppt.var_decl_lookup(tcx, VarIdent::Return).is_none() {
            panic!(
                "DATIR/decls-gen is out of sync: EXIT ppt `{base_ppt_name}:::EXIT` is \
                 missing a VariableDecl for the return value of {local_def_id:?}."
            );
        }

        let mod_path = mod_path_of(tcx, local_def_id);
        self.first_pass.observe_fn(
            mod_path,
            type_key,
            ident,
            local_def_id.to_def_id(),
            base_ppt_name,
        );
    }
}

/// Module path for `ldid`s enclosing module, joined by ::. For the crate root
/// this returns an empty string.
fn mod_path_of<'tcx>(tcx: TyCtxt<'tcx>, ldid: LocalDefId) -> ModPath {
    let parent_mod = tcx.parent_module_from_def_id(ldid);
    if parent_mod.to_local_def_id() == CRATE_DEF_ID {
        String::new()
    } else {
        tcx.def_path_str(parent_mod.to_def_id())
    }
}

/// `TypeKey` for the impl block that contains `method_ldid`. Walks the impl
/// HIR node's `self_ty` and `of_trait` paths and joins ident-only segments
/// with `::`. 
/// 
/// Returns None when the impl's self-type isn't a resolved path
/// (slice/array/tuple/ref/trait-object/fn-pointer self-types)
fn impl_type_key<'tcx>(tcx: TyCtxt<'tcx>, method_ldid: LocalDefId) -> Option<TypeKey> {
    // FIXME:  this needs to be more robust, we probably can support above types
    let impl_ldid = tcx.local_parent(method_ldid);
    let rustc_hir::Node::Item(rustc_hir::Item {
        kind:
            rustc_hir::ItemKind::Impl(rustc_hir::Impl {
                self_ty, of_trait, ..
            }),
        ..
    }) = tcx.hir_node_by_def_id(impl_ldid)
    else {
        return None;
    };

    let self_path_str = hir_ty_canonical(self_ty)?;
    let trait_path_str = match of_trait {
        Some(header) => Some(hir_path_canonical(header.trait_ref.path)?),
        None => None,
    };

    Some(match trait_path_str {
        Some(t) => TypeKey::trait_impl(self_path_str, t),
        None => TypeKey::inherent(self_path_str),
    })
}

/// HIR counterpart to `stubs.rs::ast_path_canonical`. Creates a 
/// ::-joined ident<args> form string. 
/// 
/// Returns None on non-`AngleBracketed` args, associated-type
/// constraints, const generic args, and non-path types as type args.
fn hir_path_canonical(path: &rustc_hir::Path<'_>) -> Option<String> {
    // FIXME: support above.
    let mut parts = Vec::with_capacity(path.segments.len());
    for seg in path.segments.iter() {
        parts.push(hir_segment_canonical(seg)?);
    }
    Some(parts.join("::"))
}

/// Gets the canonical representation of a single path segment.
fn hir_segment_canonical(seg: &rustc_hir::PathSegment<'_>) -> Option<String> {
    let ident = seg.ident.name.to_string();
    let Some(args) = seg.args else {
        return Some(ident);
    };

    if !args.parenthesized.eq(&rustc_hir::GenericArgsParentheses::No) {
        return None;
    }

    let mut rendered = Vec::new();
    for arg in args.args.iter() {
        let s = match arg {
            rustc_hir::GenericArg::Lifetime(lt) => lt.ident.name.to_string(),
            rustc_hir::GenericArg::Type(ty) => hir_ty_canonical(ty.as_unambig_ty())?,
            rustc_hir::GenericArg::Const(_) => panic!(
                "DATIR does not support const generic arguments in impl-block paths \
                 (encountered in segment `{}`); see hir_segment_canonical",
                seg.ident.name
            ),
            rustc_hir::GenericArg::Infer(_) => panic!(
                "DATIR does not support inferred (`_`) generic arguments in impl-block \
                 paths (encountered in segment `{}`); see hir_segment_canonical",
                seg.ident.name
            ),
        };
        rendered.push(s);
    }

    // FIXME: not really sure what to do with constraints, skipping for now.
    if !args.constraints.is_empty() {
        return None;
    }

    if rendered.is_empty() {
        Some(ident)
    } else {
        Some(format!("{ident}<{}>", rendered.join(",")))
    }
}

/// Gets the canonical represetnation of this path type
fn hir_ty_canonical(ty: &rustc_hir::Ty<'_>) -> Option<String> {
    let rustc_hir::TyKind::Path(rustc_hir::QPath::Resolved(_, path)) = ty.kind else {
        return None;
    };
    hir_path_canonical(path)
}

impl rustc_driver::Callbacks for GatherAtiInfo {
    /// Disables everything after MIR construction
    fn config(&mut self, config: &mut interface::Config) {
        config.opts.unstable_opts.no_codegen = true;
    }

    fn after_crate_root_parsing(
        &mut self,
        _compiler: &interface::Compiler,
        _krate: &mut ast::Crate,
    ) -> Compilation {
        Compilation::Continue
    }

    /// This is where the key functionality of this compiler invocation lies.
    /// Overall, the following is performed:
    ///   1. Find all locations (code spans) where:
    ///       a.
    ///   2. Find all invocations of functions that are not defined in the instrumented files
    ///      (calls to code in libraries which was left uninstrumented).
    ///
    /// As of 3/29/26, we are choosing to ignore uninstrumented libraries, meaning that
    /// (2) is really an unnecessary step. The goal is to instrument the standard library at least
    /// and after that is done, determine what needs to be added to this code to appropriately handle
    /// uninstrumented library code. The code is still left, as a proof-of-concept for later
    fn after_expansion<'tcx>(
        &mut self,
        _compiler: &interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> Compilation {
        // Iterates over all code blocks that can be invoked. This includes
        // regular functions, methods defined in impl blocks, closures, and
        // anon constants. All body owners receive a unique DefId and BodyId.
        // FIXME: This whole system needs a rework. Finding the "tracked boundary"
        // requires iterating through the entire crate (most likely file-by-file to be
        // able to differentiate where functions are defined), alongside namespace resolution
        // to differentiate TypeOne::foo from TypeTwo::foo. Is that all?
        for local_def_id in tcx.hir_body_owners() {
            let node = tcx.hir_node_by_def_id(local_def_id);

            if let rustc_hir::Node::Item(rustc_hir::Item {
                kind: rustc_hir::ItemKind::Fn { ident, .. },
                ..
            }) = node
            {
                // we found a regular function, named `ident`!
                self.record_fn(tcx, local_def_id, *ident,  None);
            } else if let rustc_hir::Node::ImplItem(rustc_hir::ImplItem {
                ident,
                kind: rustc_hir::ImplItemKind::Fn(_, _),
                ..
            }) = node
            {
                // we found a method defined in some impl block!
                let type_key = impl_type_key(tcx, local_def_id).unwrap_or_else(|| {
                    panic!(
                        "Could not derive TypeKey for impl method {local_def_id:?}, \
                         enclosing impl block has a non-path self-type."
                    )
                });
                self.record_fn(tcx, local_def_id, *ident, Some(type_key));
            } else if let rustc_hir::Node::ImplItem(_) = node {
                // non-Fn impl items (associated constants, types)
                // FIXME: probably safe to ignore for now, but should be implemented soon
            } else if let rustc_hir::Node::AnonConst(_) = node {
                // static constants, like lengths of arrays that need to be computed
                // FIXME: probably safe to ignore for now, but should be implemented soon
            } else {
                // FIXME: implement support for closures. Should closures be treated as full blown functions?
                unimplemented!(
                    "Found body owner that isn't a function while discovering ATI info: {node:#?}"
                )
            }
        }

        // at this point, self.first_pass has knowledge of every single function that
        // requires instrumentation.

        let mut find_calls_visitor = AnalyzeHirVisitor {
            tcx,
            first_pass: &mut self.first_pass,
        };
        tcx.hir_walk_toplevel_module(&mut find_calls_visitor);

        // at this point, self.first_pass has knowledge of:
        // 1. every single function that requires instrumentation to be added
        // 2. all code locations where a funciton that is not instrumented is invoked
        if self.config.print_first_pass_info {
            self.config
                .log("FirstPassInfo", format!("{:#?}", self.first_pass));
        }

        Compilation::Continue
    }

    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &interface::Compiler,
        _tcx: TyCtxt<'tcx>,
    ) -> Compilation {
        Compilation::Continue
    }
}
