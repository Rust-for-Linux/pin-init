use core::marker::PhantomPinned;
use pin_init::*;

#[pin_data]
struct Foo<'a, T: Copy, const N: usize>(&'a mut [T; N], #[pin] PhantomPinned, usize);

fn main() {}
