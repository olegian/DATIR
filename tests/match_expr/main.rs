struct MyStruct {
    x: usize,
    y: usize,
}

enum MyEnum<'a> {
    V1,
    V2(usize),
    V3(MyStruct),
    V4(&'a [usize])
}

fn main() {
    let x = MyEnum::V1;
    let y = 10;
    foo(&x, y);

    let x = MyEnum::V2(30);
    let y = 40;
    foo(&x, y);

    let x = MyEnum::V4(&[1, 2, 3]);
    let y = 20;
    bar(&x, y);

    let x = MyEnum::V3(MyStruct {
        x: 0,
        y: 1,
    });
    let y = 20;
    baz(x, y);
}

fn foo(x: &MyEnum, y: usize) -> usize {
    match x {
        MyEnum::V1 => 100,
        MyEnum::V2(x) => x + y,
        MyEnum::V3(MyStruct {
            x,
            ..
        }) => {
            x + y
        },
        MyEnum::V4(x) => {
            x.len() + y
        },
    }
}

fn bar(x: &MyEnum, y: usize) -> usize {
    match x {
        MyEnum::V1 => 100,
        MyEnum::V2(x) => x + y,
        MyEnum::V3(MyStruct {
            x,
            ..
        }) => {
            x + y
        },
        MyEnum::V4(x) => {
            x.len() + y
        },
    }
}

fn baz(mut x: MyEnum, y: usize) -> usize {
    match x {
        MyEnum::V1 => {
            y
        },
        MyEnum::V2(ref x) => {
            x + y
        },
        MyEnum::V3(ref mut my_struct) => {
            let MyStruct {
                x,
                y,
            } = my_struct;

            *x + *y
        },
        MyEnum::V4(ref slice) => {
            slice.len() + y
        },
    }
}
