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

#[pin_data]
struct GenericTuple<'a, T, const N: usize>(#[pin] CMutex<(&'a T, [u8; N])>, usize);

#[pin_data]
#[allow(dead_code)]
struct UnpinnedMutexTuple<T>(CMutex<T>, usize);

#[pin_data]
struct TupleConst<T, const N: usize>(#[pin] CMutex<[T; N]>, usize);

fn assert_unpin<T: Unpin>() {}

#[test]
fn tuple_struct_generics_are_supported() {
    let value = 77u16;
    let payload = (&value, [1, 2, 3, 4]);
    stack_pin_init!(
        let tuple = pin_init!(GenericTuple { 0 <- CMutex::new(payload), 1: 12 })
    );

    let projected = tuple.as_mut().project();
    assert_pinned_mutex(&projected.0);
    let locked = projected.0.as_ref().get_ref().lock();
    assert_eq!(*locked.0, 77u16);
    assert_eq!(locked.1, [1, 2, 3, 4]);
    assert_eq!(*projected.1, 12);
}

#[test]
fn tuple_struct_unpin_ignores_unpinned_non_unpin_field() {
    assert_unpin::<UnpinnedMutexTuple<usize>>();
}

#[test]
fn tuple_struct_const_generics_support_explicit_arguments() {
    stack_pin_init!(let tuple = pin_init!(TupleConst::<u8, 3> { 0 <- CMutex::new([1, 2, 3]), 1: 9 }));

    let projected = tuple.as_mut().project();
    assert_pinned_mutex(&projected.0);
    assert_eq!(*projected.0.as_ref().get_ref().lock(), [1, 2, 3]);
    assert_eq!(*projected.1, 9);
}

#[pin_data(PinnedDrop)]
struct DropTuple(#[pin] CMutex<usize>, usize);

static PINNED_DROP_TUPLE_DROPS: core::sync::atomic::AtomicUsize =
    core::sync::atomic::AtomicUsize::new(0);

struct DropCounter;

static FALLIBLE_TUPLE_DROPS: core::sync::atomic::AtomicUsize =
    core::sync::atomic::AtomicUsize::new(0);

impl Drop for DropCounter {
    fn drop(&mut self) {
        FALLIBLE_TUPLE_DROPS.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    }
}

fn tuple_failing_init() -> impl PinInit<TupleStruct<DropCounter>, ()> {
    // SAFETY: The closure initializes field 0, explicitly rolls it back before returning `Err`,
    // and leaves the slot otherwise untouched.
    unsafe {
        pin_init_from_closure(|slot: *mut TupleStruct<DropCounter>| {
            let field0 = core::ptr::addr_of_mut!((*slot).0);
            let init0 = CMutex::new(DropCounter);
            match init0.__pinned_init(field0) {
                Ok(()) => {}
                Err(infallible) => match infallible {},
            }
            core::ptr::drop_in_place(field0);
            Err(())
        })
    }
}

#[pinned_drop]
impl PinnedDrop for DropTuple {
    fn drop(self: Pin<&mut Self>) {
        let _ = self;
        PINNED_DROP_TUPLE_DROPS.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    }
}

#[test]
fn tuple_struct_pinned_drop_delegates_from_drop() {
    PINNED_DROP_TUPLE_DROPS.store(0, core::sync::atomic::Ordering::SeqCst);
    {
        stack_pin_init!(let _tuple = pin_init!(DropTuple { 0 <- CMutex::new(5usize), 1: 1 }));
    }
    assert_eq!(
        PINNED_DROP_TUPLE_DROPS.load(core::sync::atomic::Ordering::SeqCst),
        1
    );
}

#[test]
fn tuple_struct_fallible_init_drops_initialized_fields() {
    FALLIBLE_TUPLE_DROPS.store(0, core::sync::atomic::Ordering::SeqCst);
    stack_try_pin_init!(let tuple: TupleStruct<DropCounter> = tuple_failing_init());
    assert!(matches!(tuple, Err(())));
    assert_eq!(
        FALLIBLE_TUPLE_DROPS.load(core::sync::atomic::Ordering::SeqCst),
        1
    );
}
