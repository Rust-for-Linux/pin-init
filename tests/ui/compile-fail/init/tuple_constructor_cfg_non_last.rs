use pin_init::*;

#[pin_data]
struct Tuple(Option<i32>, i32);

fn main() {
    let _ = pin_init!(Tuple(
        #[cfg(any())]
        None,
        1
    ));
}
