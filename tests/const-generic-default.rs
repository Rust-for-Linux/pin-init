#![cfg_attr(not(RUSTC_LINT_REASONS_IS_STABLE), feature(lint_reasons))]
#![cfg_attr(not(RUSTC_RAW_REF_OP_IS_STABLE), feature(raw_ref_op))]

use pin_init::*;

#[pin_data]
struct Array<const N: usize = 0> {
    array: [u8; N],
}

#[test]
fn create_array() {
    stack_pin_init!(let array: Array<1024> = init!(Array { array <- init_zeroed() }));
    println!("{}", array.array.len());
}
