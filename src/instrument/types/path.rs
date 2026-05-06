use rustc_ast_pretty::pprust;

use crate::instrument::types::{recursively_transform_ast_type, transform_primitive};

/// Recursively transforms types nested inside a path's generic arguments.
/// If the path refers to one of the std range types, wraps the
/// whole type in `Tagged<>` so `std::ops::Range<usize>` becomes
/// `Tagged<std::ops::Range<Tagged<usize>>>`.
pub fn transform_path(target_ty: &mut rustc_ast::Ty) {
    let rustc_ast::TyKind::Path(_qself, path) = &mut target_ty.kind else {
        panic!(
            "Invoked transform_path with non-path type as input: {:?}",
            pprust::ty_to_string(target_ty)
        );
    };

    for segment in path.segments.iter_mut() {
        let Some(box ref mut arg) = segment.args else {
            continue;
        };
        match arg {
            rustc_ast::GenericArgs::AngleBracketed(rustc_ast::AngleBracketedArgs {
                args, ..
            }) => {
                for arg in args.iter_mut() {
                    match arg {
                        rustc_ast::AngleBracketedArg::Arg(generic_arg) => match generic_arg {
                            rustc_ast::GenericArg::Type(ty) => {
                                recursively_transform_ast_type(ty);
                            }
                            rustc_ast::GenericArg::Const(_)
                            | rustc_ast::GenericArg::Lifetime(_) => {}
                        },
                        rustc_ast::AngleBracketedArg::Constraint(_) => {
                            todo!("Constraint is a trait?")
                        }
                    }
                }
            }
            rustc_ast::GenericArgs::Parenthesized(rustc_ast::ParenthesizedArgs {
                inputs,
                output,
                ..
            }) => {
                for input in inputs {
                    recursively_transform_ast_type(input);
                }
                if let rustc_ast::FnRetTy::Ty(box ty) = output {
                    recursively_transform_ast_type(ty);
                }
            }
            rustc_ast::GenericArgs::ParenthesizedElided(_span) => {
                panic!("this panic is probably fine to remove")
            }
        }
    }

    // FIXME: this is not resilient to custom types that are called "Range".
    let is_range_type = path
        .segments
        .last()
        .map(|seg| {
            matches!(
                seg.ident.name.as_str(),
                "Range"
                    | "RangeInclusive"
                    | "RangeFrom"
                    | "RangeTo"
                    | "RangeToInclusive"
                    | "RangeFull"
            )
        })
        .unwrap_or(false);

    if is_range_type {
        transform_primitive(target_ty);
    }
}
