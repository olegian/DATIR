use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

#[test]
fn mut_references() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "references_mut_reborrows/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "references_mut_reborrows/main.rs::main:::EXIT",
    )));

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "references_mut_reborrows/main.rs::mut_reference_to_struct:::ENTER",
        ))
        .register("a.a", 0)
        .register("a.b.0", 1)
        .register("b", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "references_mut_reborrows/main.rs::mut_reference_to_struct:::EXIT",
        ))
        .register("a.a", 0)
        .register("a.b.0", 0)
        .register("b", 0),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "references_mut_reborrows/main.rs::reassign_mut_references:::ENTER",
        ))
        .register("a", 0)
        .register("b", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "references_mut_reborrows/main.rs::reassign_mut_references:::EXIT",
        ))
        .register("a", 0)
        .register("b", 1), // interestingly, reassinging this reference inside the inner function
                           // will mean the reference outside the function in the stub is going to
                           // still point to the original value. IS THAT CORRECT?
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "references_mut_reborrows/main.rs::reassign_through_mut_reference:::ENTER",
        ))
        .register("a", 0)
        .register("b", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "references_mut_reborrows/main.rs::reassign_through_mut_reference:::EXIT",
        ))
        .register("a", 0)
        .register("b", 0), // but assigning through the reference will actually change the referent
                           // both inside the inner function and outside.
    );

    let executable = Path::new(file!())
        .parent()
        .unwrap()
        .join("mut_references.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
