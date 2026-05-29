//! Tests that no extra warnings are emitted for non-snake-case fields when using
//! `#[pin_data]`, `init!` or `pin_init!`.
//!
//! See: https://github.com/Rust-for-Linux/pin-init/issues/125

#![deny(nonstandard_style)]
#![allow(dead_code)]
#![cfg_attr(USE_RUSTC_FEATURES, feature(lint_reasons))]
#![cfg_attr(USE_RUSTC_FEATURES, feature(raw_ref_op))]

use pin_init::*;

#[allow(non_snake_case)]
struct Foo {
    NON_STANDARD_A: usize,
    nonStandardB: Bar,
}

#[allow(non_snake_case)]
struct Bar {
    Non_Standard_C: usize,
}

impl Foo {
    fn new() -> impl Init<Self> {
        init!(Self {
            NON_STANDARD_A: {
                #[expect(
                    nonstandard_style,
                    reason = "User code warnings should not be suppressed"
                )]
                (0..2).map(|NonStandardInUserCode| NonStandardInUserCode + 1).sum()
            },
            nonStandardB <- init!(Bar { Non_Standard_C: 42 }),
        })
    }
}

// Non-camel-case struct name should not produce warnings.
#[allow(nonstandard_style)]
#[pin_data]
struct non_standard_baz {
    NON_STANDARD: usize,
    #[pin]
    nonStandardPin: usize,
}

impl non_standard_baz {
    fn new(a: impl PinInit<usize>) -> impl PinInit<Self> {
        pin_init!(Self {
            NON_STANDARD: {
                #[expect(
                    nonstandard_style,
                    reason = "User code warnings should not be suppressed"
                )]
                let NON_STANDARD_IN_USER_CODE = 41;
                NON_STANDARD_IN_USER_CODE + 1
            },
            nonStandardPin <- a,
        })
    }
}
