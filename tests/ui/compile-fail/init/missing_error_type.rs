use pin_init::*;

struct Foo {
    x: Box<usize>,
}

fn main() {
    let _ = init!(Foo { x: Box::new(0)? }?);
}
