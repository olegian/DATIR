use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

#[test]
fn multi_file_with_submodules() {
    let mut expected = ExpectedOutput::new();

    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "multi_file_with_submodules/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "multi_file_with_submodules/main.rs::main:::EXIT",
    )));

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "multi_file_with_submodules/main.rs::foo:::ENTER",
        ))
        .register("x", 0)
        .register("y", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "multi_file_with_submodules/main.rs::foo:::EXIT",
        ))
        .register("x", 0)
        .register("y", 0)
        .register("return", 0),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "multi_file_with_submodules/main.rs::inner::foo:::ENTER",
        ))
        .register("a", 0)
        .register("b", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "multi_file_with_submodules/main.rs::inner::foo:::EXIT",
        ))
        .register("a", 0)
        .register("b", 0)
        .register("return", 0),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "multi_file_with_submodules/helper.rs::helper::bar:::ENTER",
        ))
        .register("p", 0)
        .register("q", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "multi_file_with_submodules/helper.rs::helper::bar:::EXIT",
        ))
        .register("p", 0)
        .register("q", 0)
        .register("return", 0),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "multi_file_with_submodules/helper.rs::helper::nested::bar:::ENTER",
        ))
        .register("m", 0)
        .register("n", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "multi_file_with_submodules/helper.rs::helper::nested::bar:::EXIT",
        ))
        .register("m", 0)
        .register("n", 0)
        .register("return", 0),
    );

    let executable = Path::new(file!())
        .parent()
        .unwrap()
        .join("multi_file_with_submodules.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
