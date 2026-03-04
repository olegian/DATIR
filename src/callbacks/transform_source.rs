use rustc_ast::{ast, mut_visit::MutVisitor};
use rustc_driver::Compilation;
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;
use rustc_session::parse::ParseSess;

use crate::{
    file_loaders::transforming_loader::{
        FileType, Passes, TransformingFileLoader, TransformingFileLoaderConfig,
    },
    visitors::{
        AttributeExprsVisitor, TupleLiteralsVisitor, define_types_from_file
    },
};

pub struct TransformSourceCallback { }
impl TransformSourceCallback {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a> rustc_driver::Callbacks for TransformSourceCallback {
    fn config(&mut self, config: &mut interface::Config) {
        // use our custom loader to also instrument non-root files
        // this loader will be the one responsible for adding all stubs,
        // tupling all literals, etc.
        let mut passes = Passes::new();
        passes.register(Box::new(
            move |psess: &ParseSess, mut krate: &mut ast::Crate, ftype: &FileType| {
                let mut tl_vis = AttributeExprsVisitor::new();
                tl_vis.visit_crate(&mut krate);
            },
        ));

        // use custom file loader to run passes over AST before continuing compilation
        config.file_loader = Some(Box::new(TransformingFileLoader::new(
            passes,
            TransformingFileLoaderConfig::debug(),
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
