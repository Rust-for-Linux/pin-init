use pin_init::*;

#[pin_data]
struct Foo([u8; 3], #[pin] i32);

fn main() {
    let _ = init!(Foo {
        0 : [1, 2, 3],
        1 <- 42,
    });
}
