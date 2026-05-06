use crate::instrument::{instrument::InstrumentingVisitor, item::data_types};

mod addr_of;
mod array;
mod call;
mod control_flow;
mod literal;
mod ops;
mod index;
mod range;
mod assign;
mod common;

pub fn transform_expr<'session>(
    visitor: &mut InstrumentingVisitor<'session>,
    expr: &mut rustc_ast::Expr,
) {
    // on Assign(Ops?), do not walk into lhs so that the 
    // receiver does not get rewritten.
    // TODO: THIS PROBABLY HAS TO BE GUARDED BY FIRST PASS INFO STUFF
    if let rustc_ast::ExprKind::Assign(_, rhs, _) = &mut expr.kind {
        rustc_ast::mut_visit::MutVisitor::visit_expr(visitor, rhs);
        assign::transform_assign(visitor, expr);
        return;
    } else if let rustc_ast::ExprKind::AssignOp(_, _, rhs) = &mut expr.kind {
        rustc_ast::mut_visit::MutVisitor::visit_expr(visitor, rhs);
        assign::transform_assign_op(visitor, expr);
        return;
    }


    // instrument all other expressions in a post-fix order,
    // so that any inner expressions are transformed first.
    rustc_ast::mut_visit::walk_expr(visitor, expr);

    match &expr.kind {
        // handled above
        rustc_ast::ExprKind::Assign(..) => unreachable!(),
        rustc_ast::ExprKind::AssignOp(..) => unreachable!(),

        // <>
        rustc_ast::ExprKind::Lit(..) => {
            literal::transform_literal(visitor, expr);
        }

        // [ <>, <>, <> ] and [ <>; N ]
        rustc_ast::ExprKind::Array(..) | rustc_ast::ExprKind::Repeat(..) => {
            array::transform_array(visitor, expr);
        }

        // &<> and &mut <>
        rustc_ast::ExprKind::AddrOf(..) => addr_of::transform_addr_of(visitor, expr),

        // <func>(<>, <>, ...)
        rustc_ast::ExprKind::Call(..) => {
            call::transform_call(visitor, expr);
        }

        // <recv>.<method>(<>, <>, ...)
        rustc_ast::ExprKind::MethodCall(..) => {
            call::transform_method_call(visitor, expr);
        }

        // +, -, *, /, %, ||, &&, ^, &, |, <<, >>, ==, !=,  <, >, <=, >=
        rustc_ast::ExprKind::Binary(..) => {
            ops::transform_binary(visitor, expr);
        }

        // Deref, Not, Negation
        rustc_ast::ExprKind::Unary(_, _) => {
            ops::transform_unary(visitor, expr);
        }

        // if <> { <> }
        rustc_ast::ExprKind::If(..) => {
            control_flow::transform_if(visitor, expr);
        }

        // while <> { <> }
        rustc_ast::ExprKind::While(..) => {
            control_flow::transform_while(visitor, expr);
        },

        // expr in condition of if-let while-let
        rustc_ast::ExprKind::Let(..) => {
            control_flow::transform_let_condition(visitor, expr);
        },

        // for <> in <> { <> }
        rustc_ast::ExprKind::ForLoop { .. } => {
            control_flow::transform_for(visitor, expr);
        },

        // loop { <> }
        rustc_ast::ExprKind::Loop(..) => {
            control_flow::transform_loop(visitor, expr)
        },

        // match <> { <> => <> }
        rustc_ast::ExprKind::Match(..) => {
            control_flow::transform_match(visitor, expr);
        },

        // <>[<>]
        rustc_ast::ExprKind::Index(..) => {
            index::transform_index(visitor, expr);
        },

        // <>..<>
        rustc_ast::ExprKind::Range(..) => {
            range::transform_range(visitor, expr);
        },

        // |args| <body>
        rustc_ast::ExprKind::Closure(..) => {
            data_types::transform_closure(visitor, expr);
        },

        // No special transformation on the rest of these exprs
        rustc_ast::ExprKind::ConstBlock(..)
        | rustc_ast::ExprKind::Tup(..)
        | rustc_ast::ExprKind::Cast(..)
        | rustc_ast::ExprKind::Type(..)
        | rustc_ast::ExprKind::Block(..)
        | rustc_ast::ExprKind::Gen(..)
        | rustc_ast::ExprKind::Await(..)
        | rustc_ast::ExprKind::Use(..)
        | rustc_ast::ExprKind::TryBlock(..)
        | rustc_ast::ExprKind::Field(..)
        | rustc_ast::ExprKind::Underscore
        | rustc_ast::ExprKind::Path(..)
        | rustc_ast::ExprKind::Break(..)
        | rustc_ast::ExprKind::Continue(..)
        | rustc_ast::ExprKind::Ret(..)
        | rustc_ast::ExprKind::InlineAsm(..)
        | rustc_ast::ExprKind::OffsetOf(..)
        | rustc_ast::ExprKind::MacCall(..)
        | rustc_ast::ExprKind::Struct(..)
        | rustc_ast::ExprKind::Paren(..)
        | rustc_ast::ExprKind::Try(..)
        | rustc_ast::ExprKind::Yield(..)
        | rustc_ast::ExprKind::Yeet(..)
        | rustc_ast::ExprKind::Become(..)
        | rustc_ast::ExprKind::IncludedBytes(..)
        | rustc_ast::ExprKind::FormatArgs(..)
        | rustc_ast::ExprKind::UnsafeBinderCast(..)
        | rustc_ast::ExprKind::Err(..)
        | rustc_ast::ExprKind::Dummy => {}
    }
}
