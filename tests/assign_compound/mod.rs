use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

#[test]
fn assign_compound() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "assign_compound/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "assign_compound/main.rs::main:::EXIT",
    )));

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "assign_compound/main.rs::assign_tuple:::ENTER",
        ))
        .register("a.0", 0)
        .register("a.1", 1)
        .register("b.0", 2)
        .register("b.1", 3),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "assign_compound/main.rs::assign_tuple:::EXIT",
        ))
        // b is captured by value
        .register("a.0", 2)
        .register("a.1", 3),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "assign_compound/main.rs::assign_struct:::ENTER",
        ))
        .register("a.a", 0)
        .register("a.b", 1)
        .register("b.a", 2)
        .register("b.b", 3),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "assign_compound/main.rs::assign_struct:::EXIT",
        ))
        // b is captured by value
        .register("a.a", 2)
        .register("a.b", 3),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "assign_compound/main.rs::assign_enum:::ENTER",
        ))
        .register("a::StructVariant.a", 0)
        .register("a::StructVariant.b", 1)
        .register("a::TupleVariant.0", 2)
        .register("a::TupleVariant.1", 3)
        .register("b::StructVariant.a", 4)
        .register("b::StructVariant.b", 5)
        .register("b::TupleVariant.0", 6)
        .register("b::TupleVariant.1", 7),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "assign_compound/main.rs::assign_enum:::EXIT",
        ))
        // b is captured by value
        .register("a::StructVariant.a", 4)
        .register("a::StructVariant.b", 5)
        .register("a::TupleVariant.0", 6)
        .register("a::TupleVariant.1", 7),
    );

    let executable = Path::new(file!())
        .parent()
        .unwrap()
        .join("assign_compound.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
