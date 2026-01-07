use pin_init::*;

#[pin_data(Bar)]
struct Foo {}

#[pin_data(Bar)]
struct Bar();

fn main() {}
