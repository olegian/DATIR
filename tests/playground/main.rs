struct MyStruct {
    x: u32,
    y: bool,
}

enum MyEnum {
    V1(u32, u32),
    V2(f64),
}

impl MyStruct {
    fn add_to_x(&mut self, incr: u32) {
        self.x += incr;
    }
}

fn main() {
    // let a = MyEnum::V1(1, 2);
    // let b = 10; let c = 10;

    // if let MyEnum::V1(x, y) = a && b == c{
    //     assert!(true);
    // } else {
    //     assert!(false)
    // }

    baz();
}

fn bar() {
    let mut a = MyEnum::V1(1, 2);

    match a {
        MyEnum::V1(ref mut x, 10) => {
            *x += 1;
        }
        _ => {}
    }
}


fn baz() {
    let a = [1];
    let b = &mut [a; 3][1..];

    b[1] = [200];
}

// fn corge() {
//     let a = [1, 2, 3];
//     let b = [4; 100];
//     let lengths = a.len() + b.len();

//     assert_eq!(lengths, 103)
// }

fn grault(a: &mut u32, b: &&u32) {
    let tmp: u32 = **b;
    *a = tmp;
}

fn foo(a: u32, b: MyStruct, c: &MyStruct) {
    
}

