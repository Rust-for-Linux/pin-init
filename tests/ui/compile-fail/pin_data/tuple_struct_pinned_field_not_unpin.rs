use core::marker::PhantomPinned;
use pin_init::*;

#[pin_data]
struct Tuple<T>(#[pin] PhantomPinned, T);

fn assert_unpin<T: Unpin>() {}

fn main() {
    assert_unpin::<Tuple<usize>>();
}
