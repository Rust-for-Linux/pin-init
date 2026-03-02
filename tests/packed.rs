#![allow(unknown_lints)]
#![allow(unexpected_cfgs)]

use core::convert::Infallible;
use pin_init::{pin_data, pin_init};

#[pin_data]
struct MagicStruct {
    a: u32,
    b: u32,
}

#[test]
fn test_scanner_skips_unused() {
    let _ = pin_init!(MagicStruct {
        a: 1,
        b: 2,
    } ? Infallible);
}

#[test]
fn test_scanner_finds_used() {
    let _ = pin_init!(MagicStruct {
        a: 1,
        b: 2,
        _: {
            let _ = b;
        }
    } ? Infallible);
}
