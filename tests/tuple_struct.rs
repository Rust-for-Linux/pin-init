#![cfg_attr(USE_RUSTC_FEATURES, feature(lint_reasons))]

use core::ptr;

use pin_init::*;

#[pin_data]
struct TupleStruct(#[pin] i32, i32);

fn init_i32(value: i32) -> impl PinInit<i32> {
    // SAFETY: The closure always initializes `slot` with a valid `i32` value.
    unsafe {
        pin_init_from_closure(move |slot| {
            // SAFETY: `slot` is provided by the initialization framework and valid for write.
            ptr::write(slot, value);
            Ok(())
        })
    }
}

fn init_i32_unpinned(value: i32) -> impl Init<i32> {
    // SAFETY: The closure always initializes `slot` with a valid `i32` value.
    unsafe {
        init_from_closure(move |slot| {
            // SAFETY: `slot` is provided by the initialization framework and valid for write.
            ptr::write(slot, value);
            Ok(())
        })
    }
}

#[test]
#[allow(clippy::just_underscores_and_digits)]
#[allow(clippy::redundant_locals)]
fn tuple_struct_values() {
    stack_pin_init!(let foo = pin_init!(TupleStruct { 0: 42, 1: 24 }));
    assert_eq!(foo.as_ref().get_ref().0, 42);
    assert_eq!(foo.as_ref().get_ref().1, 24);
}

#[test]
#[allow(clippy::just_underscores_and_digits)]
#[allow(clippy::redundant_locals)]
fn tuple_struct_init_arrow_and_projection() {
    stack_pin_init!(let foo = pin_init!(TupleStruct { 0 <- init_i32(7), 1: 13 }));
    let mut foo = foo;
    let projected = foo.as_mut().project();
    assert_eq!(*projected._0.as_ref().get_ref(), 7);
    assert_eq!(*projected._1, 13);
}

#[test]
#[allow(clippy::just_underscores_and_digits)]
#[allow(clippy::redundant_locals)]
fn tuple_struct_constructor_form() {
    stack_pin_init!(let foo = pin_init!(TupleStruct(<- init_i32(11), 29)));
    assert_eq!(foo.as_ref().get_ref().0, 11);
    assert_eq!(foo.as_ref().get_ref().1, 29);
}

#[pin_data]
struct Triple(i32, i32, i32);

#[test]
#[allow(clippy::just_underscores_and_digits)]
fn tuple_struct_constructor_form_mixed_middle_init() {
    stack_pin_init!(let triple = pin_init!(Triple(1, <- init_i32_unpinned(2), 3)));
    assert_eq!(triple.as_ref().get_ref().0, 1);
    assert_eq!(triple.as_ref().get_ref().1, 2);
    assert_eq!(triple.as_ref().get_ref().2, 3);
}
