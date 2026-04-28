use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

#[test]
fn array_2d() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "array_high_dim/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "array_high_dim/main.rs::main:::EXIT",
    )));

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "array_high_dim/main.rs::foo:::ENTER",
        ))
        .register_array("arr", vec![3, 3, 3], 0, vec![1, 2, 3])
        .register("a", 4)
        .register("b", 5)
        .register("unused", 6),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "array_high_dim/main.rs::foo:::EXIT",
        ))
        .register_array("arr", vec![3, 3, 3], 0, vec![1, 1, 2])
        .register("a", 1)
        .register("b", 0)
        .register("unused", 6)
        .register("return", 7),
    );

    let executable = Path::new(file!())
        .parent()
        .unwrap()
        .join("array_high_dim.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
