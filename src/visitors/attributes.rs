use rustc_ast::attr::{AttrIdGenerator, mk_attr_name_value_str};
use rustc_ast::mut_visit::{self, MutVisitor};
use rustc_ast::{self as ast, AttrStyle, Safety};
use rustc_span::{DUMMY_SP, Symbol};

pub struct AttributeExprsVisitor { 
    attr_gen: AttrIdGenerator,
}

impl MutVisitor for AttributeExprsVisitor {
    /// Converts all literals into TaggedValue<T>'s
    /// while making sure those values are correctly passed
    /// between the tracked/untracked boundary.
    fn visit_expr(&mut self, expr: &mut ast::Expr) {
        mut_visit::walk_expr(self, expr);
        let id = self.attr_gen.mk_attr_id();

        expr.attrs.push(
            mk_attr_name_value_str(
                &self.attr_gen,
                AttrStyle::Outer,
                Safety::Default,
                Symbol::intern("\"my_attr\""),
                Symbol::intern(&format!("{id:?}")),
                DUMMY_SP
            )
        )
    }
}

impl AttributeExprsVisitor {
    pub fn new() -> Self {
        Self {
            attr_gen: AttrIdGenerator::new(),
        }
    }
}
