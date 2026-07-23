use core::marker::PhantomPinned;
use pin_init::*;

#[pin_data]
struct Foo<'a, T: Copy, const N: usize>(&'a mut [T; N], #[pin] PhantomPinned, usize);

fn main() {
    let mut first = [1u8, 2, 3];
    let _ = init!(Foo {
        0: &mut first,
        1: PhantomPinned,
        2 <- 10,
    });

    let mut second = [4u8, 5, 6];
    let _ = init!(Foo(&mut second, PhantomPinned, 20));
}
