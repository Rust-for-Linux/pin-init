#![allow(warnings)]

use pin_init::*;
use std::{convert::Infallible, pin::Pin, sync::Mutex};

#[pin_data]
struct SelfRef<'a> {
    #[borrows(invariant '_)]
    part: Mutex<&'outer str>,

    // In this case, the type of this field is covariant over `'a`. However, the field lifetime
    // `'outer` is invariant. Conceptually, a field's type must outlive the field lifetime, so if we
    // allow `'a` to be covariant, we can have a shortened `SelfRef<'short_a>` where `'outer`
    // outlives `'short_a`. That will be unsound.
    //
    // Therefore, we need to ensure that all invariant field lifetimes will cause the field types
    // themselves to also be invariant.
    outer: &'a String,
}

// Must fail.
fn shorten<'long: 'short, 'short>(
    x: &'long SelfRef<'long>,
    short: &'short String,
) -> &'short SelfRef<'short> {
    x
}

fn main() {}
