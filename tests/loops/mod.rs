use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

#[test]
fn loops() {
    let mut expected = ExpectedOutput::new();

    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "loops/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "loops/main.rs::main:::EXIT",
    )));

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root("loops/main.rs::if_expr:::ENTER"))
            .register("branch", 0)
            .register("a", 1)
            .register("b", 2)
            .register("c", 3)
            .register("d", 4),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root("loops/main.rs::if_expr:::EXIT"))
            .register("branch", 0)
            .register("a", 1)
            .register("b", 1)
            .register("c", 3)
            .register("d", 1)
            .register("return", 1),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "loops/main.rs::while_expr:::ENTER",
        ))
        .register("iters", 0)
        .register("a", 1)
        .register("unused", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "loops/main.rs::while_expr:::EXIT",
        ))
        .register("iters", 0)
        .register("a", 0)
        .register("unused", 2),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "loops/main.rs::loop_expr:::ENTER",
        ))
        .register("iters", 0)
        .register("a", 1)
        .register("unused", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "loops/main.rs::loop_expr:::EXIT",
        ))
        .register("iters", 0)
        .register("a", 0)
        .register("unused", 2),
    );

    let executable = Path::new(file!()).parent().unwrap().join("loops.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
