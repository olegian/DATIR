use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

#[test]
fn type_hints() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "type_hints/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "type_hints/main.rs::main:::EXIT",
    )));

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "type_hints/main.rs::struct_hints:::ENTER",
        ))
        .register("a", 0)
        .register("b", 1)
        .register("unused", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "type_hints/main.rs::struct_hints:::EXIT",
        ))
        .register("a", 0)
        .register("b", 1)
        .register("unused", 2)
        .register("return.x", 0)
        .register("return.y", 1),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "type_hints/main.rs::primitive_hints:::ENTER",
        ))
        .register("a", 0)
        .register("b", 1)
        .register("unused", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "type_hints/main.rs::primitive_hints:::EXIT",
        ))
        .register("a", 0)
        .register("b", 0)
        .register("unused", 2)
        .register("return", 0),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "type_hints/main.rs::turbofish_hints:::ENTER",
        ))
        .register("a", 0)
        .register("unused", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "type_hints/main.rs::turbofish_hints:::EXIT",
        ))
        .register("a", 0)
        .register("unused", 2)
        .register("return", 0),
    );

    let executable = Path::new(file!()).parent().unwrap().join("type_hints.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
