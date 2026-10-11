// Ensure that types that have impl that specialize on a single lifetime can be used to exploit
// pin-init.

use std::marker::PhantomData;

use pin_init::*;

struct LtSpec<'a>(PhantomData<*const &'a u32>);
struct LtSpec2<'a, 'b>(PhantomData<*const &'a &'b u32>);

unsafe impl Send for LtSpec<'static> {}
unsafe impl Sync for LtSpec<'static> {}
unsafe impl<'a> Send for LtSpec2<'a, 'a> {}
unsafe impl<'a> Sync for LtSpec2<'a, 'a> {}

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

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

fn main() {
    // All of the below checks must fail.
    assert_send::<Foo>();
    assert_sync::<Foo>();
    assert_send::<Bar>();
    assert_sync::<Bar>();
}
