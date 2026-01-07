use pin_init::*;

#[pin_data]
struct Foo {
    a: Box<Foo>
    b: Box<Foo>
}

#[pin_data]
struct Bar(Box<Bar> Box<Bar>);

fn main() {}
