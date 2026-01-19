#![cfg_attr(USE_RUSTC_FEATURES, feature(lint_reasons))]

use pin_init::{pin_data, pin_init, PinInit};

#[pin_data]
pub struct Struct {
    #[cfg(kernel)]
    field_d: Field,
    #[cfg(not(kernel))]
    field_e: Field,
}

impl Struct {
    pub fn new() -> impl PinInit<Self> {
        pin_init!(Self {
            #[cfg(kernel)]
            field_d: Field {},
            #[cfg(not(kernel))]
            field_e: Field {},
        })
    }
}

struct Field {}
