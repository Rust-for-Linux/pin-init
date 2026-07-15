// Ensure that types that have impl that specialize on a single lifetime can be used to exploit
// pin-init. Separate from selfref_lifetime_specialize.rs as somehow `Unpin` check suppresses `Send`
// and `Sync` errors.

use std::marker::PhantomData;

use pin_init::*;

struct LtSpec<'a>(PhantomData<*const &'a u32>);
struct LtSpec2<'a, 'b>(PhantomData<*const &'a &'b u32>);

#[pin_data]
struct Foo {
    lt_spec: LtSpec<'a>,
    a: u32,
}

#[pin_data]
struct Bar {
    lt_spec2: LtSpec2<'a, 'b>,
    a: u32,
    b: u32,
}

fn assert_unpin<T: Unpin>() {}

fn main() {
    // All of the below checks must fail.
    assert_unpin::<Foo>();
    assert_unpin::<Bar>();
}
