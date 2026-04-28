use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

#[test]
fn binary_search() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "binary_search/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "binary_search/main.rs::main:::EXIT",
    )));
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "binary_search/main.rs::generic_bin_search:::ENTER",
        ))
        .register_array("haystack", vec![50], 0, vec![1])
        .register("needle", 2)
        .register("lo", 3)
        .register("hi", 4),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "binary_search/main.rs::generic_bin_search:::EXIT",
        ))
        .register_array("haystack", vec![50], 0, vec![1])
        .register("needle", 0)
        .register("lo", 1)
        .register("hi", 1), // FIXME: support Option variants
                            // .register("return", 0),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "binary_search/main.rs::concrete_bin_search:::ENTER",
        ))
        .register_array("haystack", vec![25], 0, vec![1])
        .register("needle", 2)
        .register("lo", 3)
        .register("hi", 4),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "binary_search/main.rs::concrete_bin_search:::EXIT",
        ))
        .register_array("haystack", vec![25], 0, vec![1])
        .register("needle", 0)
        .register("lo", 1)
        .register("hi", 1), // FIXME: support Option variants
                            // .register("return", 0),
    );

    let executable = Path::new(file!())
        .parent()
        .unwrap()
        .join("binary_search.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
