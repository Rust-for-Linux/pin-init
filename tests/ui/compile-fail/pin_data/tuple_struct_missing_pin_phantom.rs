use pin_init::*;

#[pin_data]
struct Tuple<T>(T, core::marker::PhantomPinned);

fn main() {}
