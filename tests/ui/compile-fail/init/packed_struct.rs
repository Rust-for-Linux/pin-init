use pin_init::*;

#[repr(C, packed)]
struct Foo {
    a: i8,
    b: i32,
}

fn main() {
    let _ = init!(Foo { a: -42, b: 42 });
}
