/* Entry point file for DATIR.
 * This file creates and orchestrates the multiple compiler invocations
 * required to perform abstract type inference. The first compilation
 * gathers necessary information about the source code (namely some type
 * information), which the second compilation uses to actually mutate the
 * AST and to add in dynamic instrumentation.
*/
#![feature(rustc_private)]
#![feature(box_patterns)]
#![feature(min_specialization)]
#![feature(step_trait)]
#![feature(unsize)]
#![feature(coerce_unsized)]

extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_parse;
extern crate rustc_session;
extern crate rustc_span;
extern crate smallvec;
extern crate thin_vec;

use std::{env, sync::Arc};

use crate::args::{ArgParser, ArgSpec};
use crate::common::DatirConfig;

use decls_gen::DeclsFile;

// included so VsCode's rust-analyzer extension runs static analysis on the runtime library
mod ati;

mod args;
mod callbacks;
mod common;
mod file_loaders;
mod types;
mod visitors;

/// Entry-point. Parses DATIR's own command-line options, then forwards just
/// the source file path to each rustc compiler invocation.
pub fn main() {
    let program = env::args().next().unwrap_or_else(|| "datir".to_string());
    let parser = ArgParser::new(
        program.clone(),
        "DATIR: dynamic abstract type inference for Rust",
    )
    .arg(ArgSpec::positional(
        "file",
        "FILE",
        "Path to root source file to instrument",
    ))
    .arg(
        ArgSpec::keyword(
            "output",
            "Location of produced executable with added instrumentation",
        )
        .short("-o")
        .long("--output")
        .value_name("PATH"),
    )
    .arg(
        ArgSpec::flag(
            "release",
            "--release",
            "Run in release mode, skipping debug logging, and creating .decls file",
        )
        .short("-r"),
    )
    .arg(
        ArgSpec::flag(
            "gen-decls",
            "--gen-decls",
            "Generate a .decls file for the crate being instrumented.",
        )
        .short("-d"),
    )
    .arg(
        ArgSpec::keyword(
            "rec-depth",
            "If gen-decls is specified, optionally specifies the depth to which to recursively expand types of values at program points",
        )
        .short("-rd")
        .long("--rec-depth")
        .value_name("INT_DEPTH"),
    )
    .arg(
        ArgSpec::flag(
            "test",
            "--test",
            "Run in test mode, skipping debug logging, and using regular print ATI output",
        )
    );

    let args = parser.parse_env();

    let file_path = args
        .get_value("file")
        .expect("parser guarantees `file` is present")
        .to_string();

    let mut config = if args.is_present("release") {
        let output = std::path::Path::new(&file_path).with_extension("ati");
        DatirConfig::release(output)
    } else if args.is_present("test") {
        DatirConfig::test()
    } else {
        DatirConfig::debug()
    };

    let input_path = std::path::PathBuf::from(&file_path);
    // FIXME: it could be worth it to always generate a minimal decls with rec depth 0, just to have ppt names
    // to check against. we require that the source files are present already to be able to instrument
    // anything after all. Rethink what flags are exposed.
    config.decls_file = args.is_present("gen-decls").then(|| {
        let depth = args.get_value("rec-depth").map(|depth| {
            depth
                .parse::<usize>()
                .expect("Unable to interpret rec-depth as an integer.")
        });

        DeclsFile::from_source_file(&input_path, depth)
    });

    let mut compiler_args = vec![program, file_path];
    if let Some(output) = args.get_value("output") {
        compiler_args.push(format!("-o{output}"));
    }

    config.log("Config", format!("{:#?}", config));

    // The gather compilation
    // panics on compilation failure, therefore by the time the instrument
    // compilation starts, we know we are working with a semantically correct rust program
    let config = Arc::new(config);
    let mut gather_info = callbacks::gather_orig::GatherAtiInfo::new(config.clone());
    rustc_driver::run_compiler(&compiler_args, &mut gather_info);
    let first_pass = gather_info.into_first_pass_info();

    // The instrument compilation
    let mut cbs = callbacks::transform_ast::TransformAbstractSyntaxTreeCallbacks::new(
        first_pass,
        config.clone(),
    );
    rustc_driver::run_compiler(&compiler_args, &mut cbs);

    if args.is_present("gen-decls") {
        // output the decls file
        config
            .as_ref()
            .decls_file
            .as_ref()
            .expect("generated a decls file, but then was unable to find it for writing.")
            .write_to_file(&input_path.with_extension("decls"))
            .expect("unable to write decls file to disk");
    }
}
