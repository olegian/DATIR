struct MyTuple(u32, u32);
struct MyStruct {
    a: u32,
    b: u32,
}
enum MyEnum {
    StructVariant { a: u32, b: u32 },
    TupleVariant(u32, u32),
    UnitVariant,
}

#[ignore]
fn main() {
    let mut a = MyTuple(1, 2);
    let b = MyTuple(3, 4);
    assign_tuple(&mut a, b);

    let mut c = MyStruct { a: 1, b: 2 };
    let d = MyStruct { a: 3, b: 4 };
    assign_struct(&mut c, d);

    let mut e = MyEnum::StructVariant { a: 1, b: 2 };
    let f = MyEnum::StructVariant { a: 3, b: 4 };
    let mut g = MyEnum::TupleVariant(1, 2);
    let h = MyEnum::TupleVariant(3, 4);
    let mut i = MyEnum::UnitVariant;
    let j = MyEnum::UnitVariant;

    assign_enum(&mut e, f);
    assign_enum(&mut g, h);
    assign_enum(&mut i, j);

    // assigning between different variants
    let mut k = MyEnum::StructVariant { a: 3, b: 4 };
    let l = MyEnum::TupleVariant(3, 4);
    assign_enum(&mut k, l);
}

fn assign_struct(a: &mut MyStruct, b: MyStruct) {
    *a = b;
}

fn assign_tuple(a: &mut MyTuple, b: MyTuple) {
    *a = b;
}

fn assign_enum(a: &mut MyEnum, b: MyEnum) {
    *a = b;
}
