#![cfg_attr(feature = "alloc", feature(allocator_api))]

use core::pin::Pin;
use pin_init::*;

#[allow(unused_attributes)]
#[path = "../examples/mutex.rs"]
mod mutex;
use mutex::*;

#[pin_data]
struct TupleStruct<T>(#[pin] CMutex<T>, i32);

fn assert_pinned_mutex<T>(_: &Pin<&mut CMutex<T>>) {}

#[test]
fn tuple_struct_brace_init_and_projection() {
    stack_pin_init!(let tuple = pin_init!(TupleStruct::<usize> { 0 <- CMutex::new(7), 1: 13 }));

    let projected = tuple.as_mut().project();
    assert_pinned_mutex(&projected.0);
    assert_eq!(*projected.0.as_ref().get_ref().lock(), 7);
    assert_eq!(*projected.1, 13);
}

#[pin_data]
struct Triple(i32, i32, i32);

#[pin_data]
struct ValueTuple<T>(T, i32);

#[pin_data]
struct DualPinned<T>(#[pin] CMutex<T>, #[pin] CMutex<T>, usize);

#[test]
fn tuple_struct_constructor_form() {
    stack_pin_init!(let triple = pin_init!(Triple(11, 29, 31)));
    assert_eq!(triple.as_ref().get_ref().0, 11);
    assert_eq!(triple.as_ref().get_ref().1, 29);
    assert_eq!(triple.as_ref().get_ref().2, 31);
}

#[test]
fn tuple_struct_constructor_infers_generics() {
    stack_pin_init!(let tuple = pin_init!(ValueTuple(9u32, 6)));
    assert_eq!(tuple.as_ref().get_ref().0, 9u32);
    assert_eq!(tuple.as_ref().get_ref().1, 6);
}

#[test]
fn tuple_struct_multi_pinned_fields_projection() {
    stack_pin_init!(
        let tuple = pin_init!(DualPinned::<usize> { 0 <- CMutex::new(1), 1 <- CMutex::new(2), 2: 3 })
    );

    let projected = tuple.as_mut().project();
    assert_pinned_mutex(&projected.0);
    assert_pinned_mutex(&projected.1);

    *projected.0.as_ref().get_ref().lock() = 10;
    *projected.1.as_ref().get_ref().lock() = 20;
    *projected.2 = 30;

    assert_eq!(*tuple.as_ref().get_ref().0.lock(), 10);
    assert_eq!(*tuple.as_ref().get_ref().1.lock(), 20);
    assert_eq!(tuple.as_ref().get_ref().2, 30);
}
