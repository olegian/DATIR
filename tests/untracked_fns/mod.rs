use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

#[test]
fn untracked_fns() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "untracked_fns/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "untracked_fns/main.rs::main:::EXIT",
    )));
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "untracked_fns/main.rs::foo:::ENTER",
        ))
        .register("a", 0)
        .register("b", 1)
        .register("c", 2)
        .register("d", 3)
        .register("e", 4),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "untracked_fns/main.rs::foo:::EXIT",
        ))
        .register("a", 0)
        .register("b", 1)
        .register("c", 2)
        .register("d", 2)
        .register("e", 3)
        .register("return", 3),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "untracked_fns/main.rs::max:::ENTER",
        ))
        .register("a", 0)
        .register("b", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "untracked_fns/main.rs::max:::EXIT",
        ))
        .register("a", 0)
        .register("b", 0)
        .register("return", 0),
    );

    let executable = Path::new(file!())
        .parent()
        .unwrap()
        .join("untracked_fns.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
