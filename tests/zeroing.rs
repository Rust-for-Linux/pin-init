#![cfg_attr(not(RUSTC_LINT_REASONS_IS_STABLE), feature(lint_reasons))]
#![cfg_attr(not(RUSTC_RAW_REF_OP_IS_STABLE), feature(raw_ref_op))]

use std::marker::PhantomPinned;

use pin_init::*;

const MARKS: usize = 64;

#[pin_data]
#[derive(Zeroable)]
pub struct Foo {
    buf: [u8; 1024 * 1024],
    marks: [*mut u8; MARKS],
    pos: usize,
    #[pin]
    _pin: PhantomPinned,
}

impl Foo {
    pub fn new() -> impl PinInit<Self> {
        pin_init!(&this in Self {
            marks: {
                let ptr = this.as_ptr();
                // SAFETY: project from the NonNull<Foo> to the buf field
                let ptr = unsafe { &raw mut (*ptr).buf }.cast::<u8>();
                [ptr; MARKS]},
            ..Zeroable::init_zeroed()
        })
    }
}

#[test]
#[cfg(any(feature = "std", feature = "alloc"))]
fn test() {
    let _ = Box::pin_init(Foo::new()).unwrap();
}
