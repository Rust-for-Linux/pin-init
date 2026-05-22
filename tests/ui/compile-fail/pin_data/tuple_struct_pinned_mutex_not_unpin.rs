#![cfg_attr(feature = "alloc", feature(allocator_api))]

use pin_init::*;

#[allow(unused_attributes)]
#[path = "../../../../examples/mutex.rs"]
mod mutex;
use mutex::CMutex;

#[pin_data]
struct Tuple<T>(#[pin] CMutex<T>, usize);

fn assert_unpin<T: Unpin>() {}

fn main() {
    assert_unpin::<Tuple<usize>>();
}
