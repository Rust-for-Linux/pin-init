use pin_init::{try_init, Init};

pub struct Foo {
    x: u64,
}

fn foo() -> bool {
    false
}

fn bar() -> bool {
    true
}

impl Foo {
    pub fn new() -> impl Init<Self, ()> {
        try_init!(Self {
            _: {
                if foo() {
                    return Err(());
                }
            },
            x: 0,
            _: {
                if bar() {
                    return Err(());
                }
            }
        }? ())
    }
}
