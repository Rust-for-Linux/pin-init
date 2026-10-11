use std::marker::PhantomPinned;

use core::{
    num::{NonZeroI32, NonZeroU8, NonZeroUsize},
    ptr::NonNull,
};
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

#[test]
fn zeroed_option_runtime_values() {
    let ref_opt: Option<&u8> = zeroed();
    let mut_ref_opt: Option<&mut u8> = zeroed();
    let non_null_opt: Option<NonNull<u8>> = zeroed();
    let non_zero_unsigned: Option<NonZeroUsize> = zeroed();
    let non_zero_signed: Option<NonZeroI32> = zeroed();

    assert!(ref_opt.is_none());
    assert!(mut_ref_opt.is_none());
    assert!(non_null_opt.is_none());
    assert!(non_zero_unsigned.is_none());
    assert!(non_zero_signed.is_none());
}

fn assert_zeroable_option<T: ZeroableOption>() {
    let _: Option<T> = zeroed();
}

#[test]
fn zeroed_option_generic_compile_check() {
    assert_zeroable_option::<&u8>();
    assert_zeroable_option::<&mut u8>();
    assert_zeroable_option::<NonNull<u8>>();
    assert_zeroable_option::<NonZeroU8>();
}

#[allow(dead_code)]
#[derive(Zeroable)]
union Mixed {
    a: usize,
    b: &'static usize,
}

#[test]
fn zeroed_union_with_one_zeroable_field() {
    let value: Mixed = zeroed();
    // SAFETY: `a` is the field written by zeroing and `usize` accepts all bit patterns.
    assert_eq!(unsafe { value.a }, 0);
}

#[allow(dead_code)]
#[derive(Zeroable)]
union AllZeroable {
    a: usize,
    b: u8,
}

#[test]
fn zeroed_union_with_all_fields_zeroable() {
    let value: AllZeroable = zeroed();
    // SAFETY: `a` is the field written by zeroing and `usize` accepts all bit patterns.
    assert_eq!(unsafe { value.a }, 0);
}

#[allow(dead_code)]
#[derive(Zeroable)]
union Generic<T: Copy> {
    a: T,
    b: &'static usize,
}

fn assert_zeroable<T: Zeroable>() {}

#[test]
fn zeroed_generic_union_compile_check() {
    assert_zeroable::<Generic<usize>>();
}
