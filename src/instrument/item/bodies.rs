use rustc_ast_pretty::pprust;

use crate::gather::type_key;
use crate::instrument::{instrument::InstrumentingVisitor, types};

/// Walks the body, then wraps parameter and return types in `Tagged<T>`
/// for free functions that pass 1 observed.
pub fn transform_fn(visitor: &mut InstrumentingVisitor, fn_item: &mut rustc_ast::Item) {
    let rustc_ast::ItemKind::Fn(box rustc_ast::Fn {
        ident,
        sig: rustc_ast::FnSig { decl, .. },
        body,
        ..
    }) = &mut fn_item.kind
    else {
        return;
    };

    if visitor
        .first_pass
        .lookup_free_fn(&visitor.mod_path, ident.as_str())
        .is_none()
    {
        return;
    }

    if let Some(body) = body {
        rustc_ast::mut_visit::walk_block(visitor, body);
    }

    for param in &mut decl.inputs {
        if matches!(
            param.ty.kind,
            rustc_ast::TyKind::Ref(
                _,
                rustc_ast::MutTy {
                    mutbl: rustc_ast::Mutability::Mut,
                    ..
                }
            )
        ) {
            let rustc_ast::PatKind::Ident(mode, _, _) = &mut param.pat.kind else {
                panic!(
                    "Mut-ref parameter has non-ident pattern: {:?}",
                    pprust::pat_to_string(&param.pat)
                );
            };
            mode.1 = rustc_ast::Mutability::Mut;
        }

        types::recursively_transform_ast_type(&mut param.ty);
    }

    if let rustc_ast::FnRetTy::Ty(return_type) = &mut decl.output {
        types::recursively_transform_ast_type(return_type);
    }
}

/// Walks the body of every method that pass 1 observed in this impl,
/// then wraps parameter and return types.
pub fn transform_impl(visitor: &mut InstrumentingVisitor, impl_item: &mut rustc_ast::Item) {
    let rustc_ast::ItemKind::Impl(rustc_ast::Impl {
        of_trait,
        self_ty,
        items,
        ..
    }) = &mut impl_item.kind
    else {
        return;
    };

    let type_key =
        type_key::TypeKey::try_from_ast(of_trait.as_deref().map(|h| &h.trait_ref), self_ty)
            .unwrap_or_else(|| {
                panic!(
                    "instrumentation could not derive TypeKey from impl self-type \
                 `{}` in module `{}`; only path self/trait types are supported",
                    pprust::ty_to_string(self_ty),
                    visitor.mod_path,
                )
            });

    for assoc_item in items.iter_mut() {
        let rustc_ast::AssocItemKind::Fn(box rustc_ast::Fn {
            ident,
            sig: rustc_ast::FnSig { decl, .. },
            body,
            ..
        }) = &mut assoc_item.kind
        else {
            continue;
        };

        if visitor
            .first_pass
            .lookup_method(&visitor.mod_path, &type_key, ident.as_str())
            .is_none()
        {
            continue;
        }

        if let Some(body) = body {
            rustc_ast::mut_visit::walk_block(visitor, body);
        }

        for param in &mut decl.inputs {
            if !matches!(param.ty.peel_refs().kind, rustc_ast::TyKind::ImplicitSelf) {
                types::recursively_transform_ast_type(&mut param.ty);
            }
        }

        if let rustc_ast::FnRetTy::Ty(ret_ty) = &mut decl.output {
            types::recursively_transform_ast_type(ret_ty);
        }
    }
}

pub fn transform_trait(_visitor: &mut InstrumentingVisitor, _trait_item: &mut rustc_ast::Item) {
    // TODO: trait items aren't instrumented yet.
}
