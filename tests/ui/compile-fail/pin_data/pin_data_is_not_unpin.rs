use pin_init::*;

#[pin_data]
struct Foo {
    a: u8,
    #[pin]
    _pin: core::marker::PhantomPinned,
}

fn assert_unpin<T: core::marker::Unpin>() {}

fn main() {
    assert_unpin::<Foo>()
}
