/* This file defines the callbacks used by the second compilation, responsible
 * for actually modifying the AST to include instrumentation. Under the hood,
 * this file defines passes to run over the AST via the TransformingFileLoader,
 * so that every file being compiled gets properly instrumented, and not just the
 * crate root.
*/

use std::sync::Arc;

use rustc_ast::{ast, mut_visit::MutVisitor};
use rustc_driver::Compilation;
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;
use rustc_session::parse::ParseSess;

use crate::{
    common::DatirConfig,
    file_loaders::transforming_loader::{FileType, Passes, TransformingFileLoader},
    types::ati_info::FirstPassInfo,
    visitors::{
        AddInstrumentationVisitor, UpdateTypesVisitor, add_crate_attribute, define_types_from_file, import_root_crate
    },
};

/// Callbacks used to transform the ASTs of all files being instrumented.
pub struct TransformAbstractSyntaxTreeCallbacks {
    first_pass: Arc<FirstPassInfo>,
    config: Arc<DatirConfig>,
}

impl TransformAbstractSyntaxTreeCallbacks {
    /// Constructor
    pub fn new(first_pass: FirstPassInfo, config: Arc<DatirConfig>) -> Self {
        Self {
            first_pass: Arc::new(first_pass),
            config,
        }
    }
}

impl<'a> rustc_driver::Callbacks for TransformAbstractSyntaxTreeCallbacks {
    fn config(&mut self, config: &mut interface::Config) {
        // use our custom loader to also instrument non-root files
        // this loader will be the one responsible for adding all stubs,
        // tupling all literals, etc.

        let first_pass = self.first_pass.clone();
        let datir_config = self.config.clone();
        let mut passes = Passes::new();
        passes.register(Box::new(
            move |psess: &ParseSess,
                  mut krate: &mut ast::Crate,
                  ftype: &FileType,
                  module_path: &str| {
                // This visitor converts all expressions:
                // 1. Literals -> tracked, tagged literals (`1` -> `ATI::track(1)`)
                // 2. Arrays -> tracked, tagged arrays (`[1; 3]` -> `ATI::track_array([1; 3])`).
                //    note that the inner `1` expr would've been converted via step (1)
                // 3. Slices -> tracked, tagged slices (`&[1; 3] as [usize]` -> `ATI::track_slice(&[1; 3])`).
                // 4. If/While conditions are untupled, so they still work.
                // 5. Binary-ops / assign-ops into Block expressions that merge together appropriate tags
                // 6. Indexes in Index expressions are untupled, so the index can be used to access the collection
                let mut inst_vis = AddInstrumentationVisitor::new(&first_pass, psess);
                inst_vis.visit_crate(&mut krate);

                // discovers all functions that will be instrumented, and updates
                // the function signatures to tag all passed-in params, if necessary.
                // also updates type definitions in structs to have fields be tagged.
                let mut types_vis = UpdateTypesVisitor::new(&first_pass, module_path);
                types_vis.collect_known_idents(krate);
                types_vis.visit_crate(&mut krate);

                // create all required function stubs, which perform site management
                let stub_info = types_vis.get_fn_signatures();
                stub_info.create_stub_items(&mut krate, &psess);

                if datir_config.print_function_signatures {
                    datir_config.log("StubInfo", format!("{:#?}", stub_info));
                }

                // make the ATI types available to dependancies
                if matches!(ftype, FileType::Dep) {
                    import_root_crate(&mut krate, &psess);
                }
            },
        ));

        // use custom file loader to run passes over AST before continuing compilation
        config.file_loader = Some(Box::new(TransformingFileLoader::new(
            passes,
            self.config.clone(),
        )));
    }

    /// Define necessary types in the root file. All other files will
    /// import these types from the root.
    fn after_crate_root_parsing(
        &mut self,
        compiler: &interface::Compiler,
        krate: &mut ast::Crate,
    ) -> Compilation {
        let cwd = std::env::current_dir().unwrap();
        define_types_from_file(&cwd.join("src/ati/ati.rs"), &compiler.sess.psess, krate);
        define_types_from_file(&cwd.join("src/ati/tagged.rs"), &compiler.sess.psess, krate);
        add_crate_attribute(
            "#![feature(min_specialization)]",
            &compiler.sess.psess,
            krate,
        );

        Compilation::Continue
    }

    // leaving the other callbacks just in case they are useful
    fn after_expansion<'tcx>(
        &mut self,
        _compiler: &interface::Compiler,
        _tcx: TyCtxt<'tcx>,
    ) -> Compilation {
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
