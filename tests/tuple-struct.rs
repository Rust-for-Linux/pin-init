use pin_init::{pin_data, pin_init, PinInit};

#[pin_data]
pub struct Bar;

impl Bar {
    fn new() -> impl PinInit<Self> {
        pin_init!(Self {})
    }
}
#[pin_data]
pub struct Foo(#[pin] Bar);

impl Foo {
    fn new() -> impl PinInit<Self> {
        pin_init!(Self { 0 <- Bar::new() })
    }
}
