use pin_init::pin_data;

#[pin_data]
pub struct Bar;

#[pin_data]
pub struct Foo(#[pin] Bar);
