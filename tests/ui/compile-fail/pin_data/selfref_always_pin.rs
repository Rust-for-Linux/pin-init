// Ensure that types that self-references are always pinned.

use pin_init::*;

#[pin_data]
struct Foo {
    #[borrowed]
    f: u32,
}

#[pin_data]
struct Bar {
    b: &'f u32,
    f: u32,
}

#[pin_data]
struct Baz {
    #[borrowed]
    f: u32,
}

// Manual implementation must fail.
impl Unpin for Baz {}

fn assert_unpin<T: Unpin>() {}

fn main() {
    // All of the below checks must fail.
    assert_unpin::<Foo>();
    assert_unpin::<Bar>();
}
