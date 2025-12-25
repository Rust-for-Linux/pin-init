#![allow(dead_code)]

use pin_init::{init, Init};

struct Foo {}

struct Error;

impl Foo {
    fn new() -> impl Init<Foo, Error> {
        init!(
            #[default_error(Error)]
            Foo {}
        )
    }
}
