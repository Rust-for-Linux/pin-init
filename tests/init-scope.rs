#![allow(dead_code)]
#![cfg_attr(USE_RUSTC_FEATURES, feature(lint_reasons))]
#![cfg_attr(USE_RUSTC_FEATURES, feature(raw_ref_op))]

use pin_init::*;

#[pin_data]
pub struct MyStruct {
    a: usize,
    b: isize,
}

fn foo() -> Result<usize, ()> {
    Ok(0)
}

impl MyStruct {
    pub fn new() -> impl Init<Self, ()> {
        init_scope(|| {
            let a = foo()?;
            Ok(init!(Self { a, b: 42 }?()))
        })
    }

    pub fn new2() -> impl PinInit<Self, ()> {
        pin_init_scope(|| {
            let a = foo()?;
            Ok(pin_init!(Self { a, b: 42 }?()))
        })
    }
}
