struct Inner<'a>(&'a mut u32);

struct MyStruct<'a, 'b, 'c> {
    a: &'a mut u32,
    b: &'b mut Inner<'c>,
}

fn main() {
    let mut v1 = 10;
    let mut v2 = 20;
    reassign_mut_references(&mut v1, &mut v2);

    let mut v3 = 30;
    let v4 = 30;
    reassign_through_mut_reference(&mut v3, v4);

    let mut a = Inner(&mut v1);
    let mut b = MyStruct {
        a: &mut v2,
        b: &mut a,
    };
    let v5 = 30;
    mut_reference_to_struct(&mut b, v5);
}

fn reassign_mut_references<'a>(mut a: &'a mut u32, b: &'a mut u32) {
    a = b;
}

fn reassign_through_mut_reference(mut a: &mut u32, b: u32) {
    *a = b;
}

fn mut_reference_to_struct(a: &mut MyStruct, b: u32) {
    let tmp = b + *a.a;
    *a.b.0 = tmp;
}
