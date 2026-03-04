/* Because we are invoking the compiler multiple times, we need some
 * way of relaying information between the multiple compilations. This file
 * defines some structs which can be used for just that.
 *
 * FunctionBoundaries is used to relay information from the first pass, which
 * discovers what functions we are going to be instrumenting and where we are
 * making calls to untracked functions.
 *
 * FunctionBoundaries is then used to during the second compilation to only
 * instrument specific functions, during which FunctionSignatures is constructed.
 * FunctionSignatures is used to record the updated data types used in function
 * inputs and outputs, as well as the function name and parameter names.
 * FunctionSignatures is then consumed by the stub creation process, to add in
 * the correct stubs responsible for managing sites.
*/

use std::collections::{HashMap, HashSet};

use rustc_ast::{FieldDef, Param, ast, token};
use rustc_hir::def_id::DefId;
use rustc_middle as mir;
use rustc_session::parse::ParseSess;
use rustc_span::{Ident, Span};

use crate::common;

/// Contains all information that is going to be passed between the
/// first and second compilation rounds. Populated by invoking the
/// compiler, using the GatherAtiInfo callbacks.
#[derive(Debug)]
pub struct FunctionBoundaries {
    /// which user-defined functions are instrumented across the entire project
    tracked_fn_def_ids: HashSet<DefId>,
    tracked_fn_idents: HashSet<Ident>,

    /// places where a non-tracked function is called
    /// mapped to a string representation of the return type at that point.
    // FIXME: I'm not convinced that a string here is the best thing to store
    // but until I see an actual use for that, idc. Could be the mir::ty::Ty.
    untracked_fn_calls: HashMap<Span, String>,
}

impl FunctionBoundaries {
    pub fn new() -> Self {
        Self {
            tracked_fn_def_ids: HashSet::new(),
            tracked_fn_idents: HashSet::new(),
            untracked_fn_calls: HashMap::new(),
        }
    }

    ///////
    // Learn info

    /// register that a function with `ident` and `def_id` should
    /// later instrumented.
    pub fn observe_tracked_fn(&mut self, ident: &Ident, def_id: DefId) {
        self.tracked_fn_idents.insert(ident.clone());
        self.tracked_fn_def_ids.insert(def_id);
    }

    /// register that a function call was made to an untracked funtion at
    /// `loc`, which returned a value of type `ty`.
    pub fn observe_untracked_fn_call<'a>(&mut self, loc: Span, ty: mir::ty::Ty<'a>) {
        self.untracked_fn_calls.insert(loc, ty.to_string());
    }

    ///////
    // Use info

    /// returns true if this identifier represent a tracked function.
    pub fn is_fn_ident_tracked(&self, ident: &Ident) -> bool {
        self.tracked_fn_idents.contains(ident)
    }

    /// returns true if this def_id represents a tracked function.
    pub fn is_fn_def_id_tracked(&self, def_id: &DefId) -> bool {
        self.tracked_fn_def_ids.contains(def_id)
    }

    /// fetches the original type returned from an untracked function call,
    /// if one exists at that location.
    pub fn get_untracked_fn_call_ret_ty(&self, location: &Span) -> Option<&String> {
        self.untracked_fn_calls.get(location)
    }
}

/// This struct is responsible for packaging together the new function signatures
/// of functions that were modified, for which function stubs need to be created.
/// Each stub requires knowledge of the function name, param names + types, and the
/// return type, all of which is encoded in the `tracked` map.
#[derive(Default, Debug)]
pub struct FunctionSignatures {
    fn_sigs: HashMap<String, (Vec<ast::Param>, Option<ast::Ty>)>,
    def_structs: HashMap<String, Vec<ast::FieldDef>>,
}
impl FunctionSignatures {
    /// Constructor
    pub fn new() -> Self {
        Self {
            fn_sigs: HashMap::new(),
            def_structs: HashMap::new(),
        }
    }

    /// Observes a new struct def
    pub fn register_struct_def(&mut self, name: &str, field_defs: Vec<&FieldDef>) {
        self.def_structs
            .insert(name.into(), field_defs.into_iter().cloned().collect());
    }

    /// Observes a new function signature, with the given name, inputs, and output
    pub fn register_fn_sig(&mut self, name: &str, inputs: Vec<&Param>, output: Option<&ast::Ty>) {
        self.fn_sigs.insert(
            name.into(),
            (inputs.into_iter().cloned().collect(), output.cloned()),
        );
    }

    // might be able to have this fully consume self
    pub fn create_stub_items(&self, krate: &mut ast::Crate, psess: &ParseSess) {
        for fn_name in self.fn_sigs.keys() {
            let code = self.create_fn_stub(fn_name);

            for item in common::parse_items(psess, code, None) {
                krate.items.insert(0, item);
            }
        }
    }

    fn create_fn_stub(&self, fn_name: &str) -> String {
        let (inputs, output) = self
            .fn_sigs
            .get(fn_name)
            .expect("Attempting to create function stub out of non-registered function");

        let (declared_params, passed_params): (Vec<String>, Vec<String>) = inputs
            .iter()
            .map(|param| {
                let name = self.get_param_name(param);
                let ptype = common::get_type_string(&param.ty);

                (format!("{name}: {ptype}"), name)
            })
            .unzip();

        let enter_param_binds = self.create_site_binds("site_enter", fn_name);
        let exit_param_binds = self.create_site_binds("site_exit", fn_name);

        self.create_stub(
            fn_name,
            declared_params.join(", "),
            passed_params.join(", "),
            enter_param_binds.join("\n"),
            exit_param_binds.join("\n"),
            output.as_ref().map(|ty| common::get_type_string(ty)),
        )
    }

    fn get_param_name(&self, param: &ast::Param) -> String {
        match param.pat.kind {
            rustc_ast::PatKind::Ident(_, ident, _) => ident.as_str().to_string(),
            _ => {
                unreachable!("Cannot get name of non-Ident param name")
            }
        }
    }

    fn create_site_binds(&self, site_name: &str, fn_name: &str) -> Vec<String> {
        let (inputs, _) = self.fn_sigs.get(fn_name).unwrap();
        println!("{:?}", inputs);

        // at this point, inputs should have been wrapped in TV<> if possible
        inputs
            .iter()
            .filter(|param| {
                matches!(
                    &param.ty.kind,
                    ast::TyKind::Array(_, _)
                        | ast::TyKind::Slice(_)
                        | ast::TyKind::Ref(_, _)
                        | ast::TyKind::Tup(_)
                        | ast::TyKind::Path(_, _)
                )
            })
            .map(|param| {
                let var_name = self.get_param_name(param);
                self.create_bind_statements(site_name, var_name, &param.ty).join("\n")

                // let binds = self.get_repr_to_access_path(name, &param.ty);
                // binds.iter().map(|(repr, ap)| format!(r#"{site_name}.bind("{repr}", &{ap});"#))
                //     .collect::<Vec<_>>()
                //     .join("\n")
            })
            .collect::<Vec<String>>()

        // vec!["AAAAAAAAAAAA".into()]
    }

    fn create_bind_statements(&self, site_name: &str, name: String, ty: &ast::Ty) -> Vec<String> {
        match &ty.kind {
            ast::TyKind::Slice(ty) => {
                /*
                    my_param: &('a)?[T] 

                    for (i, p) in my_param.iter().enumerate() {
                        site.bind(format!("my_param[{i}]"), &p);
                    }
                */ 
                let aps = self.get_tracked_access_path(&name, ty);
                aps.into_iter().map(|ap| {
                    let apc = &ap;
                    format!(r#"
                        for (i, p) in {name}.iter().enumerate() {{
                            {site_name}.bind("{name}[i]{apc}", p[i]);
                        }}
                    "#)
                }).collect()
            },
            ast::TyKind::Array(ty, anon_const) => {
                let ast::ExprKind::Lit(token::Lit { kind, symbol, suffix }) = anon_const.value.kind else {
                    panic!("AAA");
                };

                let n = symbol.as_str().parse::<usize>();
                let aps = self.get_tracked_access_path(&name, ty);
                aps.into_iter().map(|ap| {
                    let apc = &ap;
                    format!(r#"
                        for i in 0..n {{
                            {site_name}.bind("{name}[i]{apc}", p[i]);
                        }}
                    "#)
                }).collect()
            },
            ast::TyKind::Tup(thin_vec) => {
                vec![]
            },
            ast::TyKind::Ptr(mut_ty) => {
                vec![]
            },
            ast::TyKind::Ref(lifetime, mut_ty) => {
                vec![]
            },
            ast::TyKind::Path(qself, path) => {
                vec![]
            },
            _ => panic!("Cannot construct bind statement for {name} with type: {ty:?}"),

            // ast::TyKind::PinnedRef(lifetime, mut_ty) => todo!(),
            // ast::TyKind::FnPtr(fn_ptr_ty) => todo!(),
            // ast::TyKind::UnsafeBinder(unsafe_binder_ty) => todo!(),
            // ast::TyKind::Never => todo!(),
            // ast::TyKind::TraitObject(generic_bounds, trait_object_syntax) => todo!(),
            // ast::TyKind::ImplTrait(node_id, generic_bounds) => todo!(),
            // ast::TyKind::Paren(ty) => todo!(),
            // ast::TyKind::Infer => todo!(),
            // ast::TyKind::ImplicitSelf => todo!(),
            // ast::TyKind::MacCall(mac_call) => todo!(),
            // ast::TyKind::CVarArgs => todo!(),
            // ast::TyKind::Pat(ty, ty_pat) => todo!(),
            // ast::TyKind::Dummy => todo!(),
            // ast::TyKind::Err(error_guaranteed) => todo!(),
        }
    }

    fn get_tracked_access_path(&self, name: &str, ty: &ast::Ty) -> Vec<String> {
        let mut res = Vec::new();
        match &ty.kind {
            ast::TyKind::Path(_, ast::Path { segments, .. }) => {
                if common::is_type_tupled_value(&ty) {
                    res.push(name.to_string());
                } else if common::is_type_tupled_array(ty) {
                    let Some(box ast::GenericArgs::AngleBracketed(ast::AngleBracketedArgs {
                        args,
                        ..
                    })) = &segments
                        .last()
                        .expect("Found Path type with no segments")
                        .args
                    else {
                        unreachable!("TaggedArray type was missing generic parameters");
                    };

                    let ast::AngleBracketedArg::Arg(ast::GenericArg::Type(box ty)) = &args[0]
                    else {
                        unreachable!("TaggedArray first param was not the array data type");
                    };

                    let ast::AngleBracketedArg::Arg(ast::GenericArg::Const(ast::AnonConst {
                        value:
                            box ast::Expr {
                                kind: ast::ExprKind::Lit(token::Lit { symbol, .. }),
                                ..
                            },
                        ..
                    })) = &args[1]
                    else {
                        unreachable!("TaggedArray first param was not the array data type");
                    };

                    let aps = self.get_tracked_access_path(name, ty);
                    let size = symbol.as_str().parse::<usize>().unwrap();
                    for i in 0..size {
                        for ap in &aps {
                            res.push(format!("{ap}.0[{i}]"))
                        }
                    }
                } else if let Some(fields) = self
                    .def_structs
                    .get(segments.iter().last().unwrap().ident.as_str())
                {
                    // Tracked structs
                    for field in fields {
                        let field_name = field
                            .ident
                            .expect("Only support named fields in structs")
                            .as_str()
                            .to_string();

                        let mut aps = self
                            .get_tracked_access_path(&field_name, &field.ty)
                            .iter()
                            .map(|ap| format!("{name}.{ap}"))
                            .collect::<Vec<_>>();

                        res.append(&mut aps);
                    }
                } else {
                    // This is where things like Vec will show up
                }
            }

            ast::TyKind::Array(ty, anon_const) => {
                let aps = self.get_tracked_access_path(name, ty);

                let ast::ExprKind::Lit(token::Lit { symbol, .. }) = anon_const.value.kind else {
                    panic!("Found array with non-const size");
                };
                let size = symbol.as_str().parse::<usize>().unwrap();
                for i in 0..size {
                    for ap in &aps {
                        res.push(format!("{ap}[{i}]"))
                    }
                }
            }

            ast::TyKind::Ref(_, ast::MutTy { ty, .. }) => {
                let aps = self.get_tracked_access_path(name, ty);
                for ap in &aps {
                    res.push(format!("&{ap}"))
                }
            }

            ast::TyKind::Tup(tys) => {
                for (i, ty) in tys.iter().enumerate() {
                    let aps = self.get_tracked_access_path(&format!("{name}.{i}"), ty);
                    for ap in aps.into_iter() {
                        res.push(ap);
                    }
                }
            }

            ast::TyKind::Slice(ty) => {
                
            },

            _ => unreachable!(),
            // ast::TyKind::Ptr(mut_ty) => todo!(),
            // ast::TyKind::PinnedRef(lifetime, mut_ty) => todo!(),
            // ast::TyKind::FnPtr(fn_ptr_ty) => todo!(),
            // ast::TyKind::UnsafeBinder(unsafe_binder_ty) => todo!(),
            // ast::TyKind::Never => todo!(),
            // ast::TyKind::TraitObject(generic_bounds, trait_object_syntax) => todo!(),
            // ast::TyKind::ImplTrait(node_id, generic_bounds) => todo!(),
            // ast::TyKind::Paren(ty) => todo!(),
            // ast::TyKind::Infer => todo!(),
            // ast::TyKind::ImplicitSelf => todo!(),
            // ast::TyKind::MacCall(mac_call) => todo!(),
            // ast::TyKind::CVarArgs => todo!(),
            // ast::TyKind::Pat(ty, ty_pat) => todo!(),
            // ast::TyKind::Dummy => todo!(),
            // ast::TyKind::Err(error_guaranteed) => todo!(),
        };

        res
    }

    fn create_stub(
        &self,
        fn_name: &str,
        declared_params: String,
        passed_params: String,
        enter_param_binds: String,
        exit_param_binds: String,
        output: Option<String>,
    ) -> String {
        if fn_name == "main" {
            // TODO: environment stuff for main
            // this is kind of a silly stub for now...
            format!(
                r#"
                pub fn main() {{
                    let mut site_enter = ATI_ANALYSIS.lock().unwrap().get_site("main::ENTER");
                    ATI_ANALYSIS.lock().unwrap().update_site(site_enter);

                    let mut site_exit = ATI_ANALYSIS.lock().unwrap().get_site("main::EXIT");
                    ATI_ANALYSIS.lock().unwrap().update_site(site_exit);

                    main_unstubbed();

                    ATI_ANALYSIS.lock().unwrap().report();
                }}
            "#
            )
        } else if let Some(ret) = output {
            // with a return value
            format!(
                r#"
                pub fn {fn_name}({declared_params}) -> {ret} {{
                    let mut site_enter = ATI_ANALYSIS.lock().unwrap().get_site("{fn_name}::ENTER");
                    {enter_param_binds}
                    ATI_ANALYSIS.lock().unwrap().update_site(site_enter);

                    let mut site_exit = ATI_ANALYSIS.lock().unwrap().get_site("{fn_name}::EXIT");
                    {exit_param_binds}
                    ATI_ANALYSIS.lock().unwrap().update_site(site_exit);

                    let res = {fn_name}_unstubbed({passed_params});

                    let mut site_exit = ATI_ANALYSIS.lock().unwrap().get_site("{fn_name}::EXIT");
                    site_exit.bind("RET", &res);
                    ATI_ANALYSIS.lock().unwrap().update_site(site_exit);

                    return res;
                }}
            "#
            )
        } else {
            // without a return value
            format!(
                r#"
                pub fn {fn_name}({declared_params}) {{
                    let mut site_enter = ATI_ANALYSIS.lock().unwrap().get_site("{fn_name}::ENTER");
                    {enter_param_binds}
                    ATI_ANALYSIS.lock().unwrap().update_site(site_enter);

                    let mut site_exit = ATI_ANALYSIS.lock().unwrap().get_site("{fn_name}::EXIT");
                    {exit_param_binds}
                    ATI_ANALYSIS.lock().unwrap().update_site(site_exit);

                    {fn_name}_unstubbed({passed_params});

                    let mut site_exit = ATI_ANALYSIS.lock().unwrap().get_site("{fn_name}::EXIT");
                    ATI_ANALYSIS.lock().unwrap().update_site(site_exit);
                }}
            "#
            )
        }
    }
}
