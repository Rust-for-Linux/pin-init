extern crate pin_init;
use pin_init::*;

#[derive(Zeroable)]
enum Num {
    A(u32),
    B(i32),
}

#[derive(MaybeZeroable)]
enum Num2 {
    A(u32),
    B(i32),
}

fn main() {}
