use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

#[test]
fn simple() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "simple/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "simple/main.rs::main:::EXIT",
    )));
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root("simple/main.rs::foo:::ENTER"))
            .register("x", 0)
            .register("y", 0)
            .register("z", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root("simple/main.rs::foo:::EXIT"))
            .register("x", 0)
            .register("y", 0)
            .register("z", 1)
            .register("return", 0),
    );

    let executable = Path::new(file!()).parent().unwrap().join("simple.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
