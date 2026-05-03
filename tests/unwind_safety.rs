//! Regression tests for:
//!
//! 1. the unwind-safety fix in `[pin_]init_array_from_fn`: a panic or error during
//!    element `i` initialization must drop the previously initialized elements `0..i`.
//! 2. the unwind-safety fix in `[pin_]chain`: a panic or error from the chained
//!    closure must drop the value initialized by the first stage.
//!
//! For more information, see: https://github.com/Rust-for-Linux/pin-init/issues/136.

#![cfg(any(feature = "std", feature = "alloc"))]
#![cfg_attr(feature = "alloc", feature(allocator_api))]

use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

use pin_init::*;

#[allow(unused_attributes)]
#[path = "../examples/error.rs"]
mod error;
use error::Error;

static DROPS: AtomicUsize = AtomicUsize::new(0);
/// Serialize tests so the shared [DROPS] counter stays meaningful even
/// when cargo runs tests in parallel.
static LOCK: Mutex<()> = Mutex::new(());

struct Counted;

impl Drop for Counted {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

fn maybe_panicking_init(should_panic: bool) -> impl Init<Counted, core::convert::Infallible> {
    // SAFETY: on `Ok(())` we have written a valid `Counted` into `slot`;
    // on panic we never wrote, so `slot` is left uninitialized as required.
    unsafe {
        init_from_closure(move |slot: *mut Counted| {
            assert!(!should_panic);
            slot.write(Counted);
            Ok(())
        })
    }
}

fn maybe_panicking_pin_init(
    should_panic: bool,
) -> impl PinInit<Counted, core::convert::Infallible> {
    // SAFETY: on `Ok(())` we have written a valid `Counted` into `slot`;
    // on panic we never wrote, so `slot` is left uninitialized as required.
    //
    // `Counted: Unpin`, so pinning invariants are trivial.
    unsafe {
        pin_init_from_closure(move |slot: *mut Counted| {
            assert!(!should_panic);
            slot.write(Counted);
            Ok(())
        })
    }
}

fn fallible_init(should_error: bool) -> impl Init<Counted, Error> {
    // SAFETY: on `Ok(())` we have written a valid `Counted` into `slot`;
    // on `Err(Error)` we never wrote, so `slot` is left uninitialized as required.
    unsafe {
        init_from_closure(move |slot: *mut Counted| {
            if should_error {
                Err(Error)
            } else {
                slot.write(Counted);
                Ok(())
            }
        })
    }
}

fn fallible_pin_init(should_error: bool) -> impl PinInit<Counted, Error> {
    // SAFETY: on `Ok(())` we have written a valid `Counted` into `slot`;
    // on `Err(Error)` we never wrote, so `slot` is left uninitialized as required.
    //
    // `Counted: Unpin`, so pinning invariants are trivial.
    unsafe {
        pin_init_from_closure(move |slot: *mut Counted| {
            if should_error {
                Err(Error)
            } else {
                slot.write(Counted);
                Ok(())
            }
        })
    }
}

#[test]
fn init_array_from_fn_drops_initialized_prefix_on_panic() {
    const N: usize = 10;
    const PANIC_AT: usize = 5;

    let _g = LOCK.lock().unwrap();
    DROPS.store(0, Ordering::SeqCst);

    let func = AssertUnwindSafe(|| {
        let init = init_array_from_fn(|i| {
            let should_panic = i == PANIC_AT;
            maybe_panicking_init(should_panic)
        });
        let _array: Result<Box<[Counted; N]>, _> = Box::init(init);
    });
    let result = catch_unwind(func);

    assert!(result.is_err());
    assert_eq!(DROPS.load(Ordering::SeqCst), PANIC_AT);
}

#[test]
fn pin_init_array_from_fn_drops_initialized_prefix_on_panic() {
    const N: usize = 10;
    const PANIC_AT: usize = 5;

    let _g = LOCK.lock().unwrap();
    DROPS.store(0, Ordering::SeqCst);

    let func = AssertUnwindSafe(|| {
        let init = pin_init_array_from_fn(|i| {
            let should_panic = i == PANIC_AT;
            maybe_panicking_pin_init(should_panic)
        });
        let _array: Result<Pin<Box<[Counted; N]>>, _> = Box::pin_init(init);
    });
    let result = catch_unwind(func);

    assert!(result.is_err());
    assert_eq!(DROPS.load(Ordering::SeqCst), PANIC_AT);
}

#[test]
fn init_array_from_fn_drops_initialized_prefix_on_error() {
    const N: usize = 10;
    const ERROR_AT: usize = 5;

    let _g = LOCK.lock().unwrap();
    DROPS.store(0, Ordering::SeqCst);

    let init = init_array_from_fn(|i| {
        let should_error = i == ERROR_AT;
        fallible_init(should_error)
    });
    let result: Result<Box<[Counted; N]>, _> = Box::try_init(init);
    assert!(result.is_err());
    assert_eq!(DROPS.load(Ordering::SeqCst), ERROR_AT);
}

#[test]
fn pin_init_array_from_fn_drops_initialized_prefix_on_error() {
    const N: usize = 10;
    const ERROR_AT: usize = 5;

    let _g = LOCK.lock().unwrap();
    DROPS.store(0, Ordering::SeqCst);

    let init = pin_init_array_from_fn(|i| {
        let should_error = i == ERROR_AT;
        fallible_pin_init(should_error)
    });
    let result: Result<Pin<Box<[Counted; N]>>, _> = Box::try_pin_init(init);
    assert!(result.is_err());
    assert_eq!(DROPS.load(Ordering::SeqCst), ERROR_AT);
}

#[test]
fn init_array_from_fn_no_double_drop_on_success() {
    const N: usize = 8;

    let _g = LOCK.lock().unwrap();
    DROPS.store(0, Ordering::SeqCst);

    {
        let init = init_array_from_fn(|_| maybe_panicking_init(false));
        let result: Result<Box<[Counted; N]>, _> = Box::init(init);
        assert!(result.is_ok());
        assert_eq!(DROPS.load(Ordering::SeqCst), 0);
    }
    assert_eq!(DROPS.load(Ordering::SeqCst), N);
}

#[test]
fn pin_init_array_from_fn_no_double_drop_on_success() {
    const N: usize = 8;

    let _g = LOCK.lock().unwrap();
    DROPS.store(0, Ordering::SeqCst);

    {
        let pin_init = pin_init_array_from_fn(|_| maybe_panicking_pin_init(false));
        let result: Result<Pin<Box<[Counted; N]>>, _> = Box::pin_init(pin_init);
        assert!(result.is_ok());
        assert_eq!(DROPS.load(Ordering::SeqCst), 0);
    }
    assert_eq!(DROPS.load(Ordering::SeqCst), N);
}

#[test]
fn chain_init_drops_first_stage_on_panic() {
    let _g = LOCK.lock().unwrap();
    DROPS.store(0, Ordering::SeqCst);

    let func = AssertUnwindSafe(|| {
        let init =
            maybe_panicking_init(false).chain(|_| -> Result<(), core::convert::Infallible> {
                panic!();
            });
        let _: Result<Box<Counted>, _> = Box::init(init);
    });
    let result = catch_unwind(func);

    assert!(result.is_err());
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
}

#[test]
fn chain_pin_init_drops_first_stage_on_panic() {
    let _g = LOCK.lock().unwrap();
    DROPS.store(0, Ordering::SeqCst);

    let func = AssertUnwindSafe(|| {
        let init = maybe_panicking_pin_init(false).pin_chain(
            |_| -> Result<(), core::convert::Infallible> {
                panic!();
            },
        );
        let _: Result<Pin<Box<Counted>>, _> = Box::pin_init(init);
    });
    let result = catch_unwind(func);

    assert!(result.is_err());
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
}

#[test]
fn chain_init_drops_first_stage_on_error() {
    let _g = LOCK.lock().unwrap();
    DROPS.store(0, Ordering::SeqCst);

    let init = fallible_init(false).chain(|_| Err(Error));
    let result: Result<Box<Counted>, _> = Box::try_init(init);
    assert!(result.is_err());
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
}

#[test]
fn chain_pin_init_drops_first_stage_on_error() {
    let _g = LOCK.lock().unwrap();
    DROPS.store(0, Ordering::SeqCst);

    let init = fallible_pin_init(false).pin_chain(|_| Err(Error));
    let result: Result<Pin<Box<Counted>>, _> = Box::try_pin_init(init);
    assert!(result.is_err());
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
}

#[test]
fn chain_init_no_double_drop_on_success() {
    let _g = LOCK.lock().unwrap();
    DROPS.store(0, Ordering::SeqCst);

    {
        let init = maybe_panicking_init(false).chain(|_| Ok(()));
        let result: Result<Box<Counted>, _> = Box::init(init);
        assert!(result.is_ok());
        assert_eq!(DROPS.load(Ordering::SeqCst), 0);
    }
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
}

#[test]
fn chain_pin_init_no_double_drop_on_success() {
    let _g = LOCK.lock().unwrap();
    DROPS.store(0, Ordering::SeqCst);

    {
        let init = maybe_panicking_pin_init(false).pin_chain(|_| Ok(()));
        let result: Result<Pin<Box<Counted>>, _> = Box::pin_init(init);
        assert!(result.is_ok());
        assert_eq!(DROPS.load(Ordering::SeqCst), 0);
    }
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
}
