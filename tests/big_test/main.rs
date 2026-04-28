mod my_vec;

// would be really cool to make this test work, on a simple
// custom vec implementation
fn main() {
    let mut v: my_vec::Vector<u32> = my_vec::Vector::new();
    v.push(10);
    v.push(20);
    v.push(30);

    foo(v, 1, 2);
}

fn foo(mut v: my_vec::Vector<u32>, a: u32, b: u32) -> u32 {
    let elem = v.pop().unwrap();
    v.push(elem + a);
    b
}
