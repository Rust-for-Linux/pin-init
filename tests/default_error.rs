#![allow(dead_code)]
#![cfg_attr(USE_RUSTC_FEATURES, feature(lint_reasons))]

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
