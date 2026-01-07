use pin_init::*;

#[pin_data]
#[pin_data]
struct Foo {
    #[pin]
    a: usize,
}

#[pin_data]
#[pin_data]
struct Bar(#[pin] usize);

fn main() {}
