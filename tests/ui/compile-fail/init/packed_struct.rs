use pin_init::*;

#[repr(C, packed)]
#[derive(Zeroable)]
struct Foo {
    a: i8,
    b: i32,
}

fn main() {
    let _ = init!(Foo { a: -42, b: 42 });
    let _ = init!(Foo {
        b: 42,
        ..Zeroable::init_zeroed()
    });
}
