#![cfg_attr(USE_RUSTC_FEATURES, feature(lint_reasons))]
#![cfg_attr(USE_RUSTC_FEATURES, feature(raw_ref_op))]
#![cfg_attr(feature = "alloc", feature(allocator_api))]

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

#[test]
fn tuple_struct_values() {
    stack_pin_init!(let foo = pin_init!(TupleStruct { 0: 42, 1: 24 }));
    assert_eq!(foo.as_ref().get_ref().0, 42);
    assert_eq!(foo.as_ref().get_ref().1, 24);
}

#[test]
#[allow(clippy::redundant_locals)]
fn tuple_struct_init_arrow_and_projection() {
    stack_pin_init!(let foo = pin_init!(TupleStruct { 0 <- init_i32(7), 1: 13 }));
    let mut foo = foo;
    let projected = foo.as_mut().project();
    assert_eq!(*projected._0.as_ref().get_ref(), 7);
    assert_eq!(*projected._1, 13);
}

#[test]
fn tuple_struct_constructor_form() {
    stack_pin_init!(let foo = pin_init!(TupleStruct(11, 29)));
    assert_eq!(foo.as_ref().get_ref().0, 11);
    assert_eq!(foo.as_ref().get_ref().1, 29);
}
