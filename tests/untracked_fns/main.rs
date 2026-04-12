#![allow(unused)]

#[ignore]
fn main() {
    foo(1, 2, 3, 4, 5);
    bar(8, 2);
}

/// tests untracked function calls
fn foo(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
    // m_ab, a, b stay in separate ATs because of fn boundary
    let m_ab = std::cmp::max(a, b);

    // m_cd, c, d all merged together by this call
    let m_cd = max(c, d);

    return m_cd;  // important that this AT is different from any other AT
}

fn max(a: u32, b: u32) -> u32 {
    if a < b { b } else { a }
}

/// tests untracked method calls
fn bar(x: u32, y: u32) -> u32 {
    let tmp1 = x.trailing_zeros();
    let tmp2 = x.pow(y);

    return tmp1 + tmp2;
}
