use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

#[test]
fn lis() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "longest_increasing_subsequence/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "longest_increasing_subsequence/main.rs::main:::EXIT",
    )));

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "longest_increasing_subsequence/main.rs::lis:::ENTER",
        ))
        .register_array("haystack", vec![20], 0, vec![1]),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "longest_increasing_subsequence/main.rs::lis:::EXIT",
        ))
        .register_array("haystack", vec![20], 0, vec![1])
        .register("return.0", 1)
        .register_array("return.1", vec![6], 0, vec![1]),
    );

    let executable = Path::new(file!()).parent().unwrap().join("lis.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
