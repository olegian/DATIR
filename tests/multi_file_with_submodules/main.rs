#![allow(unused)]

mod helper;

#[ignore]
fn main() {
    let a = foo(1, 2);
    let b = inner::foo(3, 4);
    let c = helper::bar(5, 6);
    let d = helper::nested::bar(7, 8);
}

fn foo(x: u32, y: u32) -> u32 {
    x + y
}

mod inner {
    pub fn foo(a: u32, b: u32) -> u32 {
        a + b
    }
}
