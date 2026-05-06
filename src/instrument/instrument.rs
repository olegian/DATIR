use rustc_session::parse::ParseSess;

use crate::{
    common::DatirConfig,
    gather::first_pass::FirstPassInfo,
    instrument::{expr, hoisting, item, types},
};

pub struct InstrumentingVisitor<'a> {
    pub datir_config: &'a DatirConfig,
    pub first_pass: &'a FirstPassInfo,
    pub psess: &'a ParseSess,
    pub mod_path: String,
}

impl<'a> InstrumentingVisitor<'a> {
    /// Constructor.
    pub fn new(
        psess: &'a ParseSess,
        datir_config: &'a DatirConfig,
        first_pass: &'a FirstPassInfo,
        mod_path: impl Into<String>,
    ) -> Self {
        Self {
            datir_config,
            first_pass,
            psess,
            mod_path: mod_path.into(),
        }
    }
}

impl<'a> rustc_ast::mut_visit::MutVisitor for InstrumentingVisitor<'a> {
    fn visit_param(&mut self, _node: &mut rustc_ast::Param) {}

    // stops visitor from changing any compile time constants,
    // like lengths of arrays.
    fn visit_anon_const(&mut self, _node: &mut rustc_ast::AnonConst) {}

    // transform `let x: ty` statements, into `let x: Tag(ty)`.
    fn visit_local(&mut self, local: &mut rustc_ast::Local) {
        if let Some(ty) = &mut local.ty {
            types::recursively_transform_ast_type(ty);
        }

        rustc_ast::mut_visit::walk_local(self, local);
    }

    fn visit_expr(&mut self, expr: &mut rustc_ast::Expr) {
        expr::transform_expr(self, expr);
    }
    fn visit_item(&mut self, item: &mut rustc_ast::Item) {
        item::transform_item(self, item)
    }

    // this runs last!
    fn flat_map_stmt(&mut self, stmt: rustc_ast::Stmt) -> smallvec::SmallVec<[rustc_ast::Stmt; 1]> {
        let mut stmts = rustc_ast::mut_visit::walk_flat_map_stmt(self, stmt);
        if stmts.len() != 1 {
            return stmts;
        }

        let stmt = stmts.pop().unwrap();
        hoisting::maybe_hoist_binding(self, stmt)
    }
}
