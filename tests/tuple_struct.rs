#![cfg_attr(USE_RUSTC_FEATURES, feature(lint_reasons))]
#![cfg_attr(USE_RUSTC_FEATURES, feature(raw_ref_op))]
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

#[pin_data]
struct TupleStruct<T>(#[pin] CMutex<T>, i32);

fn assert_pinned_mutex<T>(_: &Pin<&mut CMutex<T>>) {}

#[test]
fn tuple_struct_values() {
    // Baseline tuple-field syntax with index-based struct initializer.
    stack_pin_init!(let tuple = pin_init!(TupleStruct::<usize> { 0 <- CMutex::new(42), 1: 24 }));
    assert_eq!(*tuple.as_ref().get_ref().0.lock(), 42);
    assert_eq!(tuple.as_ref().get_ref().1, 24);
}

#[test]
fn tuple_struct_init_arrow_and_projection() {
    // Checks projection types and that `<-` correctly initializes the pinned tuple field.
    stack_pin_init!(let tuple = pin_init!(TupleStruct::<usize> { 0 <- CMutex::new(7), 1: 13 }));

    let projected = tuple.as_mut().project();
    assert_pinned_mutex(&projected._0);
    let projected = tuple.as_mut().project();
    assert_eq!(*projected._0.as_ref().get_ref().lock(), 7);
    assert_eq!(*projected._1, 13);
}

#[pin_data]
struct Triple(i32, i32, i32);

#[test]
fn tuple_struct_constructor_form() {
    // Constructor form remains value-only syntax.
    stack_pin_init!(let triple = pin_init!(Triple(11, 29, 31)));
    assert_eq!(triple.as_ref().get_ref().0, 11);
    assert_eq!(triple.as_ref().get_ref().1, 29);
    assert_eq!(triple.as_ref().get_ref().2, 31);
}

#[pin_data]
struct DualPinned<T>(#[pin] CMutex<T>, #[pin] CMutex<T>, usize);

#[test]
fn tuple_struct_multi_pinned_fields_projection() {
    // Both pinned tuple fields should project to `Pin<&mut CMutex<T>>` and stay usable.
    stack_pin_init!(let tuple = pin_init!(DualPinned::<usize> { 0 <- CMutex::new(1), 1 <- CMutex::new(2), 2: 3 }));
    let projected = tuple.as_mut().project();
    assert_pinned_mutex(&projected._0);
    assert_pinned_mutex(&projected._1);

    *projected._0.as_ref().get_ref().lock() = 10;
    *projected._1.as_ref().get_ref().lock() = 20;
    *projected._2 = 30;

    assert_eq!(*tuple.as_ref().get_ref().0.lock(), 10);
    assert_eq!(*tuple.as_ref().get_ref().1.lock(), 20);
    assert_eq!(tuple.as_ref().get_ref().2, 30);
}

#[test]
fn tuple_struct_generic_type_param_behavior() {
    // Keep this focused on explicit generic-arg syntax (`::<u16>`) with struct-style init.
    stack_pin_init!(let tuple = pin_init!(TupleStruct::<u16> { 0 <- CMutex::new(123u16), 1: 7 }));
    let projected = tuple.as_mut().project();
    assert_pinned_mutex(&projected._0);
    assert_eq!(*projected._0.as_ref().get_ref().lock(), 123u16);
    assert_eq!(*projected._1, 7);
}

#[pin_data]
struct ValueTuple<T>(T, i32);

#[test]
fn tuple_struct_generic_inference_constructor_form() {
    // Constructor form infers `T` from positional values.
    stack_pin_init!(let tuple = pin_init!(ValueTuple(9u32, 6)));
    assert_eq!(tuple.as_ref().get_ref().0, 9u32);
    assert_eq!(tuple.as_ref().get_ref().1, 6);
}

#[pin_data]
struct RefTuple<'a>(#[pin] CMutex<&'a usize>, usize);

#[test]
fn tuple_struct_lifetime_reference_behavior() {
    // Verifies tuple init/projection with borrowed data (`'a`) through the pinned field.
    let first = 111usize;
    let first_ref = &first;
    stack_pin_init!(let tuple = pin_init!(RefTuple { 0 <- CMutex::new(first_ref), 1: 3 }));
    assert_eq!(**tuple.as_ref().get_ref().0.lock(), 111usize);
    assert_eq!(tuple.as_ref().get_ref().1, 3);

    let second = 222usize;
    let second_ref = &second;
    stack_pin_init!(let tuple = pin_init!(RefTuple { 0 <- CMutex::new(second_ref), 1: 4 }));
    let projected = tuple.as_mut().project();
    assert_pinned_mutex(&projected._0);
    assert_eq!(**projected._0.as_ref().get_ref().lock(), 222usize);
    assert_eq!(*projected._1, 4);
}

#[test]
fn tuple_struct_projection_mutation_behavior() {
    // Confirms both projected fields can be mutated through their projected references.
    stack_pin_init!(let tuple = pin_init!(TupleStruct::<usize> { 0 <- CMutex::new(1usize), 1: 2 }));

    let projected = tuple.as_mut().project();
    *projected._0.as_ref().get_ref().lock() = 10usize;
    *projected._1 = 20;

    assert_eq!(*tuple.as_ref().get_ref().0.lock(), 10usize);
    assert_eq!(tuple.as_ref().get_ref().1, 20);
}

struct DropCounter;

static FALLIBLE_TUPLE_DROPS: AtomicUsize = AtomicUsize::new(0);

impl Drop for DropCounter {
    fn drop(&mut self) {
        FALLIBLE_TUPLE_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

fn tuple_failing_init() -> impl PinInit<TupleStruct<DropCounter>, ()> {
    // SAFETY: We emulate "initialized first field, then fail" and ensure rollback leaves no
    // partially initialized value in `slot`.
    unsafe {
        pin_init_from_closure(|slot: *mut TupleStruct<DropCounter>| {
            // Manually initialize only field 0 to model a mid-initialization failure.
            let field0 = core::ptr::addr_of_mut!((*slot).0);
            let init0 = CMutex::new(DropCounter);
            // SAFETY: `field0` points into `slot`, which is valid uninitialized memory.
            match init0.__pinned_init(field0) {
                Ok(()) => {}
                Err(infallible) => match infallible {},
            }
            // Explicit rollback is required before returning `Err` to avoid leaking initialized state.
            core::ptr::drop_in_place(field0);
            Err(())
        })
    }
}

#[test]
fn tuple_struct_fallible_init_drops_initialized_fields() {
    // A failure after partial initialization must still drop the already-initialized field.
    FALLIBLE_TUPLE_DROPS.store(0, Ordering::SeqCst);
    stack_try_pin_init!(let tuple: TupleStruct<DropCounter> = tuple_failing_init());
    assert!(matches!(tuple, Err(())));
    assert_eq!(FALLIBLE_TUPLE_DROPS.load(Ordering::SeqCst), 1);
}

#[pin_data]
struct TupleConst<T, const N: usize>(#[pin] CMutex<[T; N]>, usize);

#[test]
fn tuple_struct_const_generic_behavior() {
    // Covers tuple-field init/projection when the pinned field contains a const-generic array.
    stack_pin_init!(let tuple = pin_init!(TupleConst::<u8, 3> { 0 <- CMutex::new([1, 2, 3]), 1: 9 }));
    let projected = tuple.as_mut().project();
    assert_pinned_mutex(&projected._0);
    assert_eq!(*projected._0.as_ref().get_ref().lock(), [1, 2, 3]);
    assert_eq!(*projected._1, 9);

    stack_pin_init!(let tuple = pin_init!(TupleConst::<u8, 2> { 0 <- CMutex::new([7, 8]), 1: 5 }));
    assert_eq!(*tuple.as_ref().get_ref().0.lock(), [7, 8]);
    assert_eq!(tuple.as_ref().get_ref().1, 5);
}

#[pin_data]
struct MixedTuple<'a, T, const N: usize>(#[pin] CMutex<MixedPayload<'a, T, N>>, usize);

type MixedPayload<'a, T, const N: usize> = (&'a T, [u8; N]);

#[test]
fn tuple_struct_mixed_lifetime_type_const_generics() {
    // Stress case combining lifetime + type + const generics in one tuple pinned field.
    let value = 77u16;
    let pair = (&value, [1, 2, 3, 4]);
    stack_pin_init!(let tuple = pin_init!(MixedTuple { 0 <- CMutex::new(pair), 1: 12 }));

    let projected = tuple.as_mut().project();
    assert_pinned_mutex(&projected._0);
    let locked = projected._0.as_ref().get_ref().lock();
    assert_eq!(*locked.0, 77u16);
    assert_eq!(locked.1, [1, 2, 3, 4]);
    assert_eq!(*projected._1, 12);
}

static PINNED_DROP_TUPLE_DROPS: AtomicUsize = AtomicUsize::new(0);

#[pin_data(PinnedDrop)]
struct DropTuple(#[pin] CMutex<usize>, usize);

#[pinned_drop]
impl PinnedDrop for DropTuple {
    fn drop(self: Pin<&mut Self>) {
        let _ = self;
        PINNED_DROP_TUPLE_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn tuple_struct_pinned_drop_delegates_from_drop() {
    // `#[pin_data(PinnedDrop)]` should call our `PinnedDrop::drop` exactly once.
    PINNED_DROP_TUPLE_DROPS.store(0, Ordering::SeqCst);
    {
        stack_pin_init!(let _tuple = pin_init!(DropTuple { 0 <- CMutex::new(5usize), 1: 1 }));
    }
    assert_eq!(PINNED_DROP_TUPLE_DROPS.load(Ordering::SeqCst), 1);
}
