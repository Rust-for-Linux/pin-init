#![cfg_attr(feature = "alloc", feature(allocator_api))]

use core::{
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
};
use pin_init::*;

#[allow(unused_attributes)]
#[path = "../examples/mutex.rs"]
mod mutex;
use mutex::*;

fn assert_pinned<T>(_: &Pin<&mut T>) {}

fn assert_unpin<T: Unpin>() {}

#[pin_data]
struct TupleStruct<T>(#[pin] CMutex<T>, i32);

#[test]
fn init_and_projection() {
    stack_pin_init!(let tuple = pin_init!(TupleStruct::<usize> { 0 <- CMutex::new(7), 1: 13 }));

    let projected = tuple.project();
    assert_pinned(&projected.0);
    assert_eq!(*projected.0.as_ref().get_ref().lock(), 7);
    assert_eq!(*projected.1, 13);
}

#[pin_data]
struct Triple(i32, i32, i32);

#[test]
fn init_without_pinning() {
    stack_pin_init!(let triple = init!(Triple { 0: 37, 1: 41, 2: 43 }));

    assert_eq!(triple.as_ref().get_ref().0, 37);
    assert_eq!(triple.as_ref().get_ref().1, 41);
    assert_eq!(triple.as_ref().get_ref().2, 43);
}

#[test]
fn tuple_struct_constructor_syntax() {
    stack_pin_init!(let pinned = pin_init!(Triple(11, 29, 31)));
    stack_pin_init!(let unpinned = init!(Triple(11, 29, 31)));

    for triple in [pinned.as_ref().get_ref(), unpinned.as_ref().get_ref()] {
        assert_eq!(triple.0, 11);
        assert_eq!(triple.1, 29);
        assert_eq!(triple.2, 31);
    }
}

#[pin_data]
struct ValueTuple<T>(T, i32);

#[test]
fn tuple_struct_constructor_infers_generics() {
    stack_pin_init!(let tuple = pin_init!(ValueTuple(9u32, 6)));

    assert_eq!(tuple.as_ref().get_ref().0, 9u32);
    assert_eq!(tuple.as_ref().get_ref().1, 6);
}

#[test]
#[allow(clippy::just_underscores_and_digits)]
fn tuple_struct_constructor_does_not_shadow_numeric_identifiers() {
    let _0 = 6;
    stack_pin_init!(let tuple = pin_init!(ValueTuple(9u32, _0)));

    assert_eq!(tuple.as_ref().get_ref().1, 6);
}

#[pin_data]
struct DualPinned<T>(#[pin] CMutex<T>, #[pin] CMutex<T>, usize);

#[test]
fn multi_pinned() {
    stack_pin_init!(
        let tuple = pin_init!(DualPinned::<usize> { 0 <- CMutex::new(1), 1 <- CMutex::new(2), 2: 3 })
    );

    let projected = tuple.as_mut().project();
    assert_pinned(&projected.0);
    assert_pinned(&projected.1);

    *projected.0.as_ref().get_ref().lock() = 10;
    *projected.1.as_ref().get_ref().lock() = 20;
    *projected.2 = 30;

    assert_eq!(*tuple.as_ref().get_ref().0.lock(), 10);
    assert_eq!(*tuple.as_ref().get_ref().1.lock(), 20);
    assert_eq!(tuple.as_ref().get_ref().2, 30);
}

#[pin_data]
struct GenericTuple<'a, T, const N: usize>(#[pin] CMutex<(&'a T, [u8; N])>, usize);

#[test]
fn generics() {
    let value = 77u16;
    let payload = (&value, [1, 2, 3, 4]);
    stack_pin_init!(
        let tuple = pin_init!(GenericTuple { 0 <- CMutex::new(payload), 1: 12 })
    );

    let projected = tuple.as_mut().project();
    assert_pinned(&projected.0);
    let locked = projected.0.as_ref().get_ref().lock();
    assert_eq!(*locked.0, 77u16);
    assert_eq!(locked.1, [1, 2, 3, 4]);
    assert_eq!(*projected.1, 12);
}

#[pin_data]
struct TupleConst<T, const N: usize>(#[pin] CMutex<[T; N]>, usize);

#[test]
fn const_generics_turbofish() {
    stack_pin_init!(let tuple = pin_init!(TupleConst::<u8, 3> { 0 <- CMutex::new([1, 2, 3]), 1: 9 }));

    let projected = tuple.as_mut().project();
    assert_pinned(&projected.0);
    assert_eq!(*projected.0.as_ref().get_ref().lock(), [1, 2, 3]);
    assert_eq!(*projected.1, 9);
}

#[pin_data]
#[allow(dead_code)]
struct UnpinnedMutexTuple<T>(CMutex<T>, usize);

#[test]
fn unpin_ignores_unpinned_non_unpin_field() {
    assert_unpin::<UnpinnedMutexTuple<usize>>();
}

#[pin_data(PinnedDrop)]
struct DropTuple(#[pin] CMutex<usize>, usize);

static PINNED_DROP_TUPLE_DROPS: AtomicUsize = AtomicUsize::new(0);

#[pinned_drop]
impl PinnedDrop for DropTuple {
    fn drop(self: Pin<&mut Self>) {
        PINNED_DROP_TUPLE_DROPS.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn pinned_drop_delegates_from_drop() {
    PINNED_DROP_TUPLE_DROPS.store(0, Ordering::Relaxed);
    {
        stack_pin_init!(let _tuple = pin_init!(DropTuple { 0 <- CMutex::new(5usize), 1: 1 }));
    }
    assert_eq!(PINNED_DROP_TUPLE_DROPS.load(Ordering::Relaxed), 1);
}

static FALLIBLE_TUPLE_DROPS: AtomicUsize = AtomicUsize::new(0);

struct DropCounter;

impl Drop for DropCounter {
    fn drop(&mut self) {
        FALLIBLE_TUPLE_DROPS.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Debug)]
struct InitError;

impl From<core::convert::Infallible> for InitError {
    fn from(error: core::convert::Infallible) -> Self {
        match error {}
    }
}

fn fail<T>() -> impl Init<T, InitError> {
    // SAFETY: The closure returns an error without touching the slot.
    unsafe { init_from_closure(|_| Err(InitError)) }
}

fn tuple_failing_init() -> impl PinInit<TupleStruct<DropCounter>, InitError> {
    pin_init!(TupleStruct {
        0 <- CMutex::new(DropCounter),
        1 <- fail(),
    }? InitError)
}

#[test]
fn fallible_init_drops_initialized_fields() {
    FALLIBLE_TUPLE_DROPS.store(0, Ordering::Relaxed);
    stack_try_pin_init!(let tuple: TupleStruct<DropCounter> = tuple_failing_init());
    assert!(matches!(tuple, Err(InitError)));
    assert_eq!(
        FALLIBLE_TUPLE_DROPS.load(core::sync::atomic::Ordering::Relaxed),
        1
    );
}
