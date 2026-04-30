use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

#[test]
fn op_through_trait() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "op_through_trait/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "op_through_trait/main.rs::main:::EXIT",
    )));

    // FIXME: make specifying tuples easier
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::foo:::ENTER",
        ))
        .register("a.0", 0)
        .register("a.1", 1)
        .register("a.2", 2)

        .register("b.0", 3)
        .register("b.1", 4)
        .register("b.2", 5)

        .register("c.0", 6)
        .register("c.1", 7)
        .register("c.2", 8),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::foo:::EXIT",
        ))
        // a, b, c Captured by value.
        // .register("a.0", 0)
        // .register("a.1", 1)
        // .register("a.2", 2)

        // .register("b.0", 0)
        // .register("b.1", 1)
        // .register("b.2", 2)

        // .register("c.0", 6)
        // .register("c.1", 7)
        // .register("c.2", 8)

        .register("return.0", 0)
        .register("return.1", 1)
        .register("return.2", 2),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::bar:::ENTER",
        ))
        .register("a.a", 0)
        .register("a.b", 1)
        .register("a.c", 2)

        .register("b.a", 3)
        .register("b.b", 4)
        .register("b.c", 5)

        .register("c.a", 6)
        .register("c.b", 7)
        .register("c.c", 8),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::bar:::EXIT",
        ))
        // a, b, c, are captured by value.
        // .register("a.a", 0)
        // .register("a.b", 1)
        // .register("a.c", 2)

        // .register("b.a", 0)
        // .register("b.b", 1)
        // .register("b.c", 2)

        // .register("c.a", 6)
        // .register("c.b", 7)
        // .register("c.c", 8)

        .register("return.a", 0)
        .register("return.b", 1)
        .register("return.c", 2),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::baz:::ENTER",
        ))
        .register("a.a", 0)
        .register("a.b.0", 1)
        .register("a.b.1", 2)
        .register("a.b.2", 3)

        .register("b.a", 4)
        .register("b.b.0", 5)
        .register("b.b.1", 6)
        .register("b.b.2", 7)

        .register("c.a", 8)
        .register("c.b.0", 9)
        .register("c.b.1", 10)
        .register("c.b.2", 11),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::baz:::EXIT",
        ))
        // a, b, c captured by value
        // .register("a.a", 0)
        // .register("a.b.0", 1)
        // .register("a.b.1", 2)
        // .register("a.b.2", 3)

        // .register("b.a", 0)
        // .register("b.b.0", 1)
        // .register("b.b.1", 2)
        // .register("b.b.2", 3)

        // .register("c.a", 8)
        // .register("c.b.0", 9)
        // .register("c.b.1", 10)
        // .register("c.b.2", 11)

        .register("return.a", 0)
        .register("return.b.0", 1)
        .register("return.b.1", 2)
        .register("return.b.2", 3),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::quux:::ENTER",
        ))
        .register("a.a", 0)
        .register("a.b", 1)

        .register("b.a", 2)
        .register("b.b", 3)
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::quux:::EXIT",
        ))
        // a, b, captured by value.
        // .register("a.a", 0)
        // .register("a.b", 1)

        // .register("b.a", 0)
        // .register("b.b", 1)

        .register("return.a", 0)
        .register("return.b", 1)
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::<NewTuple as std::ops::Add>::add:::ENTER",
        ))
        .register("self.0", 0)
        .register("self.1", 1)
        .register("self.2", 2)

        .register("rhs.0", 0)  // we use this + op multiple times, on enter then
        .register("rhs.1", 1)  //  we should see corresponding self/rhs already partitioned
        .register("rhs.2", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::<NewTuple as std::ops::Add>::add:::EXIT",
        ))
        // self and rhs are captured by value
        // .register("self.0", 0)
        // .register("self.1", 1)
        // .register("self.2", 2)

        // .register("rhs.0", 0)
        // .register("rhs.1", 1)
        // .register("rhs.2", 2)

        .register("return.0", 0)
        .register("return.1", 1)
        .register("return.2", 2),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::<NewStruct as std::ops::Mul>::mul:::ENTER"
        ))
        .register("self.a", 0)
        .register("self.b", 1)
        .register("self.c", 2)

        .register("rhs.a", 3)  // but we only use this * op once... so diff sets.
        .register("rhs.b", 4)
        .register("rhs.c", 5),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::<NewStruct as std::ops::Mul>::mul:::EXIT"
        ))
        // self and rhs are captured by value, dropped on exit
        // .register("self.a", 0)
        // .register("self.b", 1)
        // .register("self.c", 2)

        // .register("rhs.a", 0)
        // .register("rhs.b", 1)
        // .register("rhs.c", 2)

        .register("return.a", 0)
        .register("return.b", 1)
        .register("return.c", 2),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::<Nested as std::ops::Add>::add:::ENTER"
        ))
        .register("self.a", 0)
        .register("self.b.0", 1)
        .register("self.b.1", 2)
        .register("self.b.2", 3)

        .register("rhs.a", 4)
        .register("rhs.b.0", 5)
        .register("rhs.b.1", 6)
        .register("rhs.b.2", 7),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::<Nested as std::ops::Add>::add:::EXIT"
        ))
        // self and rhs are captured by value,
        // and therefore dropped on exit.

        // .register("self.a", 0)
        // .register("self.b.0", 1)
        // .register("self.b.1", 2)
        // .register("self.b.2", 3)

        // .register("rhs.a", 0)
        // .register("rhs.b.0", 1)
        // .register("rhs.b.1", 2)
        // .register("rhs.b.2", 3)

        .register("return.a", 0)
        .register("return.b.0", 1)
        .register("return.b.1", 2)
        .register("return.b.2", 3),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::<WithGeneric<T> as std::ops::Add>::add:::ENTER"
        ))
        .register("self.a", 0)
        .register("self.b", 1)

        .register("rhs.a", 2)
        .register("rhs.b", 3)
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "op_through_trait/main.rs::<WithGeneric<T> as std::ops::Add>::add:::EXIT"
        ))
        // self and rhs are captured by value
        // .register("self.a", 0)
        // .register("self.b", 1)

        // .register("rhs.a", 0)
        // .register("rhs.b", 1)

        .register("return.a", 0)
        .register("return.b", 1)
    );

    let executable = Path::new(file!())
        .parent()
        .unwrap()
        .join("op_through_trait.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
