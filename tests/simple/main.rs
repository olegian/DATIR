fn main() {
    let x = 1;
    let y = 2;
    let z = 3;
    // nothing has interacted
    foo(x, y, z);
    // 1 and 2 have interacted,
    foo(x, y, z);
    // 1 and 2 have interacted,
    // in this invocation, we have that parameter foo::x
    // was previously assigned x at the start, therefore
    // the AT of foo::x was the leader of 1's interaction
    // set, which has previously interacted with 2.
    // the AT of foo::y was the leader of 2's interaction set,
    // which previously interacted with 1, (so foo::x and foo::y are
    // in the same AT).
    // During this call, foo::x is assigned 3, which makes the AT of
    // foo::x represented by leader of the 1/2 interaction set,
    // unioned with the leader of 3's interaction set, which means
    // all vars are in the same AT.
    // foo(z, x, z);
    // 1 and 2 have interacted,
    // 1 and 3 have interacted,
}

fn foo(x: u32, y: u32, z: u32) -> u32 {
    let tmp = x + y;

    tmp
}
