use std::path::Path;

use crate::common::{ExpectedOutput, ExpectedSite, compile_and_execute, delete, verify};

#[test]
fn untracked_fns() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new("main::ENTER"));
    expected.register_site(ExpectedSite::new("main::EXIT"));
    expected.register_site(
        ExpectedSite::new("foo::ENTER")
            .register("a", 0)
            .register("b", 1)
            .register("c", 2)
            .register("d", 3)
            .register("e", 4),
    );
    expected.register_site(
        ExpectedSite::new("foo::EXIT")
            .register("a", 0)
            .register("b", 1)
            .register("c", 2)
            .register("d", 2)
            .register("e", 3)
            .register("RET", 4),
    );
    expected.register_site(
        ExpectedSite::new("max::ENTER")
            .register("a", 0)
            .register("b", 1),
    );
    expected.register_site(
        ExpectedSite::new("max::EXIT")
            .register("a", 0)
            .register("b", 0)
            .register("RET", 0),
    );

    expected.register_site(
        ExpectedSite::new("bar::ENTER")
            .register("x", 0)
            .register("y", 1),
    );
    expected.register_site(
        ExpectedSite::new("bar::EXIT")
            .register("x", 0)
            .register("y", 1)
            .register("RET", 2),
    );

    let executable = Path::new(file!())
        .parent()
        .unwrap()
        .join("untracked_fns.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
