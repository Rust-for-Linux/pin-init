use pinned_init::{init, InPlaceInit, Init};

#[derive(Debug)]
struct Foo {
    a: i32,
}

impl Foo {
    fn new(val: &i32) -> impl Init<Self> + use<'_> {
        init!(Self { a: *val })
    }
}

#[derive(Debug)]
struct Bar {
    foo: Foo,
}

impl Bar {
    fn new(foo: impl Init<Foo>) -> impl Init<Self> {
        init!(Self { foo <- foo })
    }
}

fn main() {
    // problematic:
    let foo;
    {
        let val = 42;
        foo = Foo::new(&val);
    }
    let bar = Box::init(Bar::new(foo));
    println!("{bar:?}");

    // okay:
    let val = 42;
    let foo = Foo::new(&val);
    let bar = Box::init(Bar::new(foo));
    println!("{bar:?}");
}
