use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

// TODO: probably a good idea to make sites qualify the file name they are in too
#[test]
fn multi_file() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "multi_file/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "multi_file/main.rs::main:::EXIT",
    )));
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "multi_file/main.rs::foo:::ENTER",
        ))
        .register("x", 0)
        .register("y", 1)
        .register("unused", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root("multi_file/main.rs::foo:::EXIT"))
            .register("x", 0)
            .register("y", 0)
            .register("unused", 1)
            .register("return", 0),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "multi_file/dep.rs::dep::foo:::ENTER",
        ))
        .register("x", 0)
        .register("y", 1)
        .register("z", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "multi_file/dep.rs::dep::foo:::EXIT",
        ))
        .register("x", 0)
        .register("y", 1)
        .register("z", 1)
        .register("return", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "multi_file/dep.rs::dep::foo0:::ENTER",
        ))
        .register("x", 0)
        .register("y", 1)
        .register("z", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "multi_file/dep.rs::dep::foo0:::EXIT",
        ))
        .register("x", 0)
        .register("y", 0)
        .register("z", 1)
        .register("return", 0),
    );

    let executable = Path::new(file!()).parent().unwrap().join("multi_file.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
