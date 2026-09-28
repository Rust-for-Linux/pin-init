extern crate pin_init;
use pin_init::*;

use Zeroable as MyZeroable;

#[derive(Zeroable)]
struct Foo {
    a: usize,
    b: usize,
}

fn main() {
    let _ = init!(Foo {
        a: 0,
        ..MyZeroable::init_zeroed()
    });
}
