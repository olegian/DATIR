enum MyEnum {
    V1(u32, u32),
    V2(f64),
}

fn main() {
    let a = MyEnum::V1(1, 2);
    let b = 10; let c = 10;

    if let MyEnum::V1(x, y) = a && b == c{
        assert!(true);
    } else {
        assert!(false)
    }
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


