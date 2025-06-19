use pin_init::{init, Init};

pub struct Foo {
    x: u64,
    y: u64,
}

impl Foo {
    pub fn new() -> impl Init<Self> {
        init!(Self {
            _: {
                let z = 42;
            },
            x: z,
            y: z,
        })
    }
}
