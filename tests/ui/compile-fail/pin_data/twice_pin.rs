use pin_init::*;

#[pin_data]
struct Foo {
    #[pin]
    #[pin]
    a: usize,
}

fn main() {
    let _ = pin_init!(Foo { a: 1 });
}
