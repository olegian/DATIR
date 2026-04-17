fn main() {
    let n: usize = 5;
    sum_range(0, n, 0, 99);
    sum_range_inclusive(0, n, 0, 99);

    // Iterator::sum — exercises Tagged<Range>::Iterator.
    pass_range(1..5, 99);
    // inherent Tagged::len() — wrapper len returned as Tagged<usize>.
    get_length(1..100, 1);
    // DoubleEndedIterator::rev + Iterator::sum.
    reverse_sum(1..5);
    // Iterator::count.
    count_elements(1..5);
    // FusedIterator (via .fuse()) + Iterator::next.
    fused_next(1..3);
    // ExactSizeIterator::len (trait method, distinct from inherent).
    exact_size(1..5);
    // RangeBounds::start_bound / end_bound.
    check_bounds(1..5);

    let arr = &[1; 10];
    index_with_range(arr, 1, 5);

    // slice_and_modify([2; 10], 0..5, 3);
}

fn sum_range(lo: usize, hi: usize, mut acc: usize, unused: usize) -> usize {
    for i in lo..hi {
        acc = acc + i;
    }
    acc
}

fn sum_range_inclusive(lo: usize, hi: usize, mut acc: usize, unused: usize) -> usize {
    for i in lo..=hi {
        acc = acc + i;
    }
    acc
}

fn get_length(range: std::ops::Range<usize>, a: usize) -> usize {
    a + range.len()
}

fn pass_range(range: std::ops::Range<usize>, unused: usize) -> usize{
    let sum: usize = range.sum();
    sum
}

fn reverse_sum(range: std::ops::Range<usize>) -> usize {
    range.rev().sum()
}

fn count_elements(range: std::ops::Range<usize>) {
    let _n = range.count();
}

fn fused_next(range: std::ops::Range<usize>) -> usize {
    range.fuse().next().unwrap()
}

fn exact_size(range: std::ops::Range<usize>) {
    let _n = std::iter::ExactSizeIterator::len(&range);
}

fn check_bounds(range: std::ops::Range<usize>) {
    use std::ops::RangeBounds;
    let _s = range.start_bound();
    let _e = range.end_bound();
}

fn index_with_range<'a>(arr: &'a [u32; 10], lo: usize, hi: usize) -> &'a [u32] {
    &arr[lo..hi]
}

// fn slice_and_modify(mut arr: [u32; 10], range: std::ops::Range<usize>, value: u32) -> [u32; 10] {
//     for i in range {
//         arr[i] = value;
//     }
//     arr
// }
