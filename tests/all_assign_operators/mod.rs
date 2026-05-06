use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

#[test]
fn assign_operators() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "all_assign_operators/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "all_assign_operators/main.rs::main:::EXIT",
    )));

    register_arith_sites_for("addassign", &mut expected);
    register_arith_sites_for("subassign", &mut expected);
    register_arith_sites_for("mulassign", &mut expected);
    register_arith_sites_for("divassign", &mut expected);
    register_arith_sites_for("remassign", &mut expected);

    register_arith_sites_for("bitxorassign", &mut expected);
    register_arith_sites_for("bitandassign", &mut expected);
    register_arith_sites_for("bitorassign", &mut expected);

    register_shift_sites_for("shlassign", &mut expected);
    register_shift_sites_for("shrassign", &mut expected);

    let executable = Path::new(file!()).parent().unwrap().join("assign_ops.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}

fn register_arith_sites_for(op_name: &str, expected: &mut ExpectedOutput) {
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(&format!(
            "all_assign_operators/main.rs::{op_name}:::ENTER"
        )))
        .register("x", 0)
        .register("y", 1)
        .register("z", 2),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(&format!(
            "all_assign_operators/main.rs::{op_name}:::EXIT"
        )))
        .register("x", 0)
        .register("y", 0)
        .register("z", 1)
        .register("return", 1),
    );
}

/// Shift compound-assigns only merge `self` with the result; the shift
/// amount (right operand) is not unioned with the shifted value.
fn register_shift_sites_for(op_name: &str, expected: &mut ExpectedOutput) {
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(&format!(
            "all_assign_operators/main.rs::{op_name}:::ENTER"
        )))
        .register("x", 0)
        .register("y", 1)
        .register("z", 2),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(&format!(
            "all_assign_operators/main.rs::{op_name}:::EXIT"
        )))
        .register("x", 0)
        .register("y", 1)
        .register("z", 2)
        .register("return", 2),
    );
}
