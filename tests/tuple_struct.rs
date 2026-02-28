#![cfg_attr(USE_RUSTC_FEATURES, feature(lint_reasons))]

use pin_init::*;

#[pin_data]
struct TupleStruct(#[pin] i32, i32);

#[test]
#[allow(clippy::just_underscores_and_digits)]
pub fn tuple_struct() {
    stack_pin_init!(let foo = pin_init!(TupleStruct { 0: 42, 1: 24 }));
    assert_eq!(foo.as_ref().get_ref().0, 42);
    assert_eq!(foo.as_ref().get_ref().1, 24);
}
