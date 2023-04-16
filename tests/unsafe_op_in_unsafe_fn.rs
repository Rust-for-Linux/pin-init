#![deny(unsafe_op_in_unsafe_fn)]

use pin_init::*;

#[pin_init]
struct NeedPin {
    #[pin]
    a: usize,
    #[pin]
    b: usize,
}

fn main() {
    init_stack!(p = specify_err::<_, (), _>(init_pin!(NeedPin { a <- 1, b: 2 })));
    let p = p.unwrap();
    let _ = p;
}
