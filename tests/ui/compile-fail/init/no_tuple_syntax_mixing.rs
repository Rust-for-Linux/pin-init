use pin_init::*;

#[pin_data]
struct Tuple(#[pin] i32, i32);

fn main() {
    let _ = pin_init!(Tuple (0, 1: 24));
}
