extern crate pin_init;
use pin_init::*;

#[derive(Zeroable)]
union Foo {
    a: &'static usize,
    b: core::num::NonZeroUsize,
}

fn main() {}
