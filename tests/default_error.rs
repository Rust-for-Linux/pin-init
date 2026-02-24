#![allow(dead_code)]
#![cfg_attr(not(RUSTC_LINT_REASONS_IS_STABLE), feature(lint_reasons))]

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
