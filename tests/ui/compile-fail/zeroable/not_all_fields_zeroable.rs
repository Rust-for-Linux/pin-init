extern crate pinned_init;
use pinned_init::*;

#[derive(Zeroable)]
struct Foo {
    a: usize,
    b: &'static Foo,
}

fn main() {}
