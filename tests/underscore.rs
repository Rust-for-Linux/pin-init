#![cfg_attr(not(RUSTC_RAW_REF_OP_IS_STABLE), feature(raw_ref_op))]

use pin_init::{init, Init};

pub struct Foo {
    x: u64,
}

fn foo() -> bool {
    false
}

fn bar() -> bool {
    true
}

fn baz() -> Result<(), ()> {
    Err(())
}

impl Foo {
    pub fn new() -> impl Init<Self, ()> {
        init!(Self {
            _: {
                if foo() {
                    return Err(());
                }
            },
            x: 0,
            _: {
                if bar() {
                    return Err(());
                }
            }
        }? ())
    }

    pub fn create(x: u64) -> impl Init<Self, ()> {
        init!(Self {
            _: { baz()? },
            x,
        }? ())
    }
}
