use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

// FIXME: This test is currently unused, as it tests functionality
// that should be fixed when the std lib is instrumented.
fn collections() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "collections/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "collections/main.rs::main:::EXIT",
    )));

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "collections/main.rs::foo:::ENTER",
        ))
        .register("x", 0)
        .register("y", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "collections/main.rs::foo:::EXIT",
        ))
        .register("x", 0)
        .register("y", 0)
        .register("return", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "collections/main.rs::bar:::ENTER",
        ))
        .register("a", 0)
        .register("b", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "collections/main.rs::bar:::EXIT",
        ))
        .register("a", 0)
        .register("b", 0)
        .register("return", 0),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "collections/main.rs::baz:::ENTER",
        ))
        .register("a", 0)
        .register("b", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "collections/main.rs::baz:::EXIT",
        ))
        .register("a", 0)
        .register("b", 0),
    );

    let executable = Path::new(file!()).parent().unwrap().join("collections.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}

// TODO:
// 1. Delete files at start of each test
// 2. Fix unit tests not always running.
