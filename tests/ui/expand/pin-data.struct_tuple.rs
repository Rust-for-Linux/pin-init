use pin_init::*;

#[pin_data]
struct Foo([u8; 1024 * 1024], #[pin] i32);
