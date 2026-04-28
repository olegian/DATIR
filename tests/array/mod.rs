use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

#[test]
fn array() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "array/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "array/main.rs::main:::EXIT",
    )));
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root("array/main.rs::foo:::ENTER"))
            .register_array("arr", vec![3], 0, vec![1])
            .register("x", 2)
            .register("y", 3)
            .register("unused", 4),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root("array/main.rs::foo:::EXIT"))
            .register_array("arr", vec![3], 0, vec![1])
            .register("x", 0)
            .register("y", 0)
            .register("unused", 4)
            .register("return", 0),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root("array/main.rs::bar:::ENTER"))
            .register_array("arr", vec![3], 0, vec![1])
            .register("unused", 2)
            .register("y", 3)
            .register("z", 4),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root("array/main.rs::bar:::EXIT"))
            .register_array("arr", vec![3], 0, vec![1])
            .register("unused", 2)
            .register("y", 1)
            .register("z", 1)
            .register("return", 0),
    );

    let executable = Path::new(file!()).parent().unwrap().join("array.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
