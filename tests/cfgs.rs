#![cfg_attr(not(RUSTC_LINT_REASONS_IS_STABLE), feature(lint_reasons))]
#![cfg_attr(not(RUSTC_RAW_REF_OP_IS_STABLE), feature(raw_ref_op))]

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
