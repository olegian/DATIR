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
    gather::analyze_hir::AnalyzeHirVisitor,
    gather::first_pass::{FirstPassInfo, ModPath},
    gather::type_key::TypeKey,
};

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
        let body = tcx.hir_body_owned_by(local_def_id);
        for param in body.params.iter() {
            let formal = param
                .pat
                .simple_ident()
                .unwrap_or_else(|| {
                    panic!("Formal parameter of `{base_ppt_name}` is not a simple ident pattern.")
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

    fn find_instrumented_functions<'tcx>(&mut self, tcx: rustc_middle::ty::TyCtxt<'tcx>) {
        for local_def_id in tcx.hir_body_owners() {
            let node = tcx.hir_node_by_def_id(local_def_id);
            match node {
                rustc_hir::Node::Item(rustc_hir::Item {
                    kind: rustc_hir::ItemKind::Fn { ident, .. },
                    ..
                }) => {
                    self.record_fn(tcx, local_def_id, *ident, None);
                }

                rustc_hir::Node::ImplItem(rustc_hir::ImplItem {
                    ident,
                    kind: rustc_hir::ImplItemKind::Fn(_, _),
                    ..
                }) => {
                    let type_key = TypeKey::try_from_hir(tcx, local_def_id).unwrap_or_else(|| {
                        panic!(
                            "Could not derive TypeKey for impl method {local_def_id:?}, \
                            enclosing impl block has a non-path self-type."
                        )
                    });

                    self.record_fn(tcx, local_def_id, *ident, Some(type_key));
                }

                // All other items should just be ignored, we are just
                // collecting the set of functions that will get dedicated
                // program points.
                rustc_hir::Node::Item(..)
                | rustc_hir::Node::ImplItem(..)
                | rustc_hir::Node::Param(..)
                | rustc_hir::Node::ForeignItem(..)
                | rustc_hir::Node::TraitItem(..)
                | rustc_hir::Node::Variant(..)
                | rustc_hir::Node::Field(..)
                | rustc_hir::Node::AnonConst(..)
                | rustc_hir::Node::ConstBlock(..)
                | rustc_hir::Node::ConstArg(..)
                | rustc_hir::Node::Expr(..)
                | rustc_hir::Node::ExprField(..)
                | rustc_hir::Node::ConstArgExprField(..)
                | rustc_hir::Node::Stmt(..)
                | rustc_hir::Node::PathSegment(..)
                | rustc_hir::Node::Ty(..)
                | rustc_hir::Node::AssocItemConstraint(..)
                | rustc_hir::Node::TraitRef(..)
                | rustc_hir::Node::OpaqueTy(..)
                | rustc_hir::Node::TyPat(..)
                | rustc_hir::Node::Pat(..)
                | rustc_hir::Node::PatField(..)
                | rustc_hir::Node::PatExpr(..)
                | rustc_hir::Node::Arm(..)
                | rustc_hir::Node::Block(..)
                | rustc_hir::Node::LetStmt(..)
                | rustc_hir::Node::Ctor(..)
                | rustc_hir::Node::Lifetime(..)
                | rustc_hir::Node::GenericParam(..)
                | rustc_hir::Node::Crate(..)
                | rustc_hir::Node::Infer(..)
                | rustc_hir::Node::WherePredicate(..)
                | rustc_hir::Node::PreciseCapturingNonLifetimeArg(..)
                | rustc_hir::Node::Synthetic
                | rustc_hir::Node::Err(..) => {}
            }
        }
    }
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
        self.find_instrumented_functions(tcx);

        let mut find_calls_visitor = AnalyzeHirVisitor {
            tcx,
            first_pass: &mut self.first_pass,
        };
        tcx.hir_walk_toplevel_module(&mut find_calls_visitor);

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