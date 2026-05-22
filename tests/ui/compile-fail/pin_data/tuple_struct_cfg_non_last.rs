use pin_init::*;

#[pin_data]
struct Tuple(#[cfg(any())] HiddenField, i32);

fn main() {}
