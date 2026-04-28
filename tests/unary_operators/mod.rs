use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

#[test]
fn unary_operators() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "unary_operators/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "unary_operators/main.rs::main:::EXIT",
    )));

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "unary_operators/main.rs::negation:::ENTER",
        ))
        .register("x", 0)
        .register("y", 1)
        .register("z", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "unary_operators/main.rs::negation:::EXIT",
        ))
        .register("x", 0)
        .register("y", 0)
        .register("z", 1)
        .register("return", 0),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "unary_operators/main.rs::boolean_not:::ENTER",
        ))
        .register("x", 0)
        .register("y", 1)
        .register("z", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "unary_operators/main.rs::boolean_not:::EXIT",
        ))
        .register("x", 0)
        .register("y", 0)
        .register("z", 0)
        .register("return", 0),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "unary_operators/main.rs::dereference:::ENTER",
        ))
        .register("x", 0)
        .register("y", 1)
        .register("z", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "unary_operators/main.rs::dereference:::EXIT",
        ))
        .register("x", 0)
        .register("y", 1)
        .register("z", 1),
    );

    let executable = Path::new(file!()).parent().unwrap().join("unary_ops.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
