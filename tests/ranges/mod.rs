use std::path::Path;

use crate::common::{ExpectedOutput, ExpectedSite, compile_and_execute, delete, verify};

// Exercises for-loop iteration over a range. The range construction
// `lo..hi` must union lo's and hi's tags, and every yielded `i` must share
// that same AT. The `acc = acc + i` accumulation then folds acc (and the
// returned value) into the same AT. `unused` never interacts with the
// range, so it stays isolated.
//
// At ENTER, lo/hi/unused are three distinct ATs — the range hasn't been
// constructed yet. At EXIT (observed after the inner call returns),
// lo/hi/RET have unified through the range+accumulator chain.
#[test]
fn ranges() {
    let mut expected = ExpectedOutput::new();
    expected.register_site(ExpectedSite::new("main:::ENTER"));
    expected.register_site(ExpectedSite::new("main:::EXIT"));
    expected.register_site(
        ExpectedSite::new("sum_range:::ENTER")
            .register("lo", 1)
            .register("hi", 2)
            .register("acc", 3)
            .register("unused", 4),
    );
    expected.register_site(
        ExpectedSite::new("sum_range:::EXIT")
            .register("lo", 1)
            .register("hi", 1)
            .register("acc", 1)
            .register("unused", 2)
            .register("RET", 1),
    );
    // Same shape as sum_range — lo..=hi shares the AT with every yielded i,
    // the accumulator, and RET. Verifies that RangeInclusive participates in
    // the same tracked-range pipeline as HalfOpen.
    expected.register_site(
        ExpectedSite::new("sum_range_inclusive:::ENTER")
            .register("lo", 1)
            .register("hi", 2)
            .register("acc", 3)
            .register("unused", 4),
    );
    expected.register_site(
        ExpectedSite::new("sum_range_inclusive:::EXIT")
            .register("lo", 1)
            .register("hi", 1)
            .register("acc", 1)
            .register("unused", 2)
            .register("RET", 1),
    );

    // pass_range — exercises Iterator::sum on a Tagged range. The range's AT
    // covers the wrapper, both endpoints, and everything yielded during
    // iteration; `unused` stays isolated.
    expected.register_site(
        ExpectedSite::new("pass_range:::ENTER")
            .register("range", 1)
            .register("range.start", 1)
            .register("range.end", 1)
            .register("unused", 2),
    );
    expected.register_site(
        ExpectedSite::new("pass_range:::EXIT")
            .register("range", 1)
            .register("range.start", 1)
            .register("range.end", 1)
            .register("unused", 2),
    );

    // get_length — inherent Tagged::len() returns a Tagged<usize> carrying the
    // range's wrapper id; `a + range.len()` unifies `a` with the range and
    // propagates to RET.
    expected.register_site(
        ExpectedSite::new("get_length:::ENTER")
            .register("range", 1)
            .register("range.start", 1)
            .register("range.end", 1)
            .register("a", 2),
    );
    expected.register_site(
        ExpectedSite::new("get_length:::EXIT")
            .register("range", 1)
            .register("range.start", 1)
            .register("range.end", 1)
            .register("a", 1)
            .register("RET", 1),
    );

    // reverse_sum — DoubleEndedIterator::rev + Iterator::sum folds everything
    // back into the range's AT.
    expected.register_site(
        ExpectedSite::new("reverse_sum:::ENTER")
            .register("range", 1)
            .register("range.start", 1)
            .register("range.end", 1),
    );
    expected.register_site(
        ExpectedSite::new("reverse_sum:::EXIT")
            .register("range", 1)
            .register("range.start", 1)
            .register("range.end", 1)
            .register("RET", 1),
    );

    // count_elements — `.count()` is an untracked call returning raw usize;
    // its result gets a fresh AT that doesn't touch the range.
    expected.register_site(
        ExpectedSite::new("count_elements:::ENTER")
            .register("range", 1)
            .register("range.start", 1)
            .register("range.end", 1),
    );
    expected.register_site(
        ExpectedSite::new("count_elements:::EXIT")
            .register("range", 1)
            .register("range.start", 1)
            .register("range.end", 1),
    );

    // fused_next — `.fuse().next().unwrap()` yields a Tagged<usize> carrying
    // the range's wrapper id, so RET joins the range's AT.
    expected.register_site(
        ExpectedSite::new("fused_next:::ENTER")
            .register("range", 1)
            .register("range.start", 1)
            .register("range.end", 1),
    );
    expected.register_site(
        ExpectedSite::new("fused_next:::EXIT")
            .register("range", 1)
            .register("range.start", 1)
            .register("range.end", 1)
            .register("RET", 1),
    );

    // exact_size — ExactSizeIterator::len(&range) is an untracked call. The
    // &range arg stays referenced (not untupled), so the range's AT is
    // preserved; the returned raw usize gets a fresh AT.
    expected.register_site(
        ExpectedSite::new("exact_size:::ENTER")
            .register("range", 1)
            .register("range.start", 1)
            .register("range.end", 1),
    );
    expected.register_site(
        ExpectedSite::new("exact_size:::EXIT")
            .register("range", 1)
            .register("range.start", 1)
            .register("range.end", 1),
    );

    // check_bounds — RangeBounds::start_bound / end_bound. Results are raw
    // `Bound<&T>` references, so no AT folding happens.
    expected.register_site(
        ExpectedSite::new("check_bounds:::ENTER")
            .register("range", 1)
            .register("range.start", 1)
            .register("range.end", 1),
    );
    expected.register_site(
        ExpectedSite::new("check_bounds:::EXIT")
            .register("range", 1)
            .register("range.start", 1)
            .register("range.end", 1),
    );

    expected.register_site(
        ExpectedSite::new("index_with_range:::ENTER")
            .register_array("arr", vec![10], 0, vec![1])
            .register("lo", 2)
            .register("hi", 3),
    );
    expected.register_site(
        ExpectedSite::new("index_with_range:::EXIT")
            .register_array("arr", vec![10], 0, vec![1])
            .register("lo", 1)
            .register("hi", 1)
            .register_array("RET", vec![4], 0, vec![1])
    );

    let executable = Path::new(file!()).parent().unwrap().join("ranges.out");
    delete(&executable);

    let ati_output = compile_and_execute(&executable);
    verify(&ati_output, expected.inner());
}
