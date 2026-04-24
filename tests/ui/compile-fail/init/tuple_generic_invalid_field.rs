use pin_init::*;

#[pin_data]
struct Tuple<T, const N: usize>(#[pin] [T; N], i32);

fn main() {
    let _ = pin_init!(Tuple::<u8, 3> { 0: [1, 2, 3], 1: 2, 2: 3 });
}
