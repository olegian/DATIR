use std::path::Path;

use crate::common::{
    ExpectedOutput, ExpectedSite, compile_and_execute, delete, prefix_with_path_from_root, verify,
};

#[test]
fn ranges() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "ranges/main.rs::main:::ENTER",
    )));
    expected.register_site(ExpectedSite::new(prefix_with_path_from_root(
        "ranges/main.rs::main:::EXIT",
    )));
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::sum_range:::ENTER",
        ))
        .register("lo", 1)
        .register("hi", 2)
        .register("acc", 3)
        .register("unused", 4),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::sum_range:::EXIT",
        ))
        .register("lo", 1)
        .register("hi", 1)
        .register("acc", 1)
        .register("unused", 2)
        .register("return", 1),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::sum_range_inclusive:::ENTER",
        ))
        .register("lo", 1)
        .register("hi", 2)
        .register("acc", 3)
        .register("unused", 4),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::sum_range_inclusive:::EXIT",
        ))
        .register("lo", 1)
        .register("hi", 1)
        .register("acc", 1)
        .register("unused", 2)
        .register("return", 1),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::pass_range:::ENTER",
        ))
        .register("range", 1)
        .register("range.start", 1)
        .register("range.end", 1)
        .register("unused", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::pass_range:::EXIT",
        ))
        .register("range", 1)
        .register("range.start", 1)
        .register("range.end", 1)
        .register("unused", 2)
        .register("return", 1),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::get_length:::ENTER",
        ))
        .register("range", 1)
        .register("range.start", 1)
        .register("range.end", 1)
        .register("a", 2),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::get_length:::EXIT",
        ))
        .register("range", 1)
        .register("range.start", 1)
        .register("range.end", 1)
        .register("a", 1)
        .register("return", 1),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::reverse_sum:::ENTER",
        ))
        .register("range", 1)
        .register("range.start", 1)
        .register("range.end", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::reverse_sum:::EXIT",
        ))
        .register("range", 1)
        .register("range.start", 1)
        .register("range.end", 1)
        .register("return", 1),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::count_elements:::ENTER",
        ))
        .register("range", 1)
        .register("range.start", 1)
        .register("range.end", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::count_elements:::EXIT",
        ))
        .register("range", 1)
        .register("range.start", 1)
        .register("range.end", 1),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::fused_next:::ENTER",
        ))
        .register("range", 1)
        .register("range.start", 1)
        .register("range.end", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::fused_next:::EXIT",
        ))
        .register("range", 1)
        .register("range.start", 1)
        .register("range.end", 1)
        .register("return", 1),
    );

    // exact_size disabled - see tests/ranges/main.rs for rationale (UFCS
    // `ExactSizeIterator::len(&range)` incompatible with owned-TaggedRef
    // rewrite; Iterator supertrait can't be ported to TaggedRef).

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::check_bounds:::ENTER",
        ))
        .register("range", 1)
        .register("range.start", 1)
        .register("range.end", 1),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::check_bounds:::EXIT",
        ))
        .register("range", 1)
        .register("range.start", 1)
        .register("range.end", 1),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::index_with_range:::ENTER",
        ))
        .register_array("arr", vec![10], 0, vec![1])
        .register("lo", 2)
        .register("hi", 3),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::index_with_range:::EXIT",
        ))
        .register_array("arr", vec![10], 0, vec![1])
        .register("lo", 1)
        .register("hi", 1)
        .register_array("return", vec![4], 0, vec![1]),
    );

    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::slice_and_modify:::ENTER",
        ))
        .register_array("arr", vec![10], 0, vec![1])
        .register("range", 2)
        .register("range.start", 2)
        .register("range.end", 2)
        .register("value", 3),
    );
    expected.register_site(
        ExpectedSite::new(prefix_with_path_from_root(
            "ranges/main.rs::slice_and_modify:::EXIT",
        ))
        .register_array("arr", vec![10], 0, vec![1])
        .register("range", 1)
        .register("range.start", 1)
        .register("range.end", 1)
        .register("value", 0)
        .register_array("return", vec![10], 0, vec![1]), // should they all be in the same AT?
    );

    let executable = Path::new(file!()).parent().unwrap().join("ranges.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
