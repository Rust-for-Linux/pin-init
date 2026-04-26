//! Regression tests for unwind safety of `[pin_]init_array_from_fn` and `[pin_]chain`. See
//! https://github.com/Rust-for-Linux/pin-init/issues/136.

use core::{
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};
use std::pin::Pin;

use pin_init::*;

fn try_stack_init<T, E>(init: impl Init<T, E>) -> Result<T, E> {
    let mut value = MaybeUninit::uninit();
    // SAFETY: `value` provides a valid uninitialized slot, and on error we return without touching
    // the slot further.
    unsafe { init.__init(value.as_mut_ptr())? };
    // SAFETY: `value` is initialized.
    Ok(unsafe { value.assume_init() })
}

fn stack_init<T>(init: impl Init<T>) -> T {
    match try_stack_init(init) {
        Ok(value) => value,
        Err(err) => match err {},
    }
}

#[pin_data(PinnedDrop)]
struct CountDrop<'a> {
    counter: &'a AtomicUsize,
}

#[pinned_drop]
impl<'a> PinnedDrop for CountDrop<'a> {
    fn drop(self: Pin<&mut Self>) {
        self.counter.fetch_add(1, Ordering::Relaxed);
    }
}

fn maybe_panicking_init(counter: &AtomicUsize, should_panic: bool) -> impl Init<CountDrop<'_>> {
    init!(CountDrop {
        _: {
            assert!(!should_panic);
        },
        counter,
    })
}

fn maybe_panicking_pin_init(
    counter: &AtomicUsize,
    should_panic: bool,
) -> impl PinInit<CountDrop<'_>> {
    pin_init!(CountDrop {
        _: {
            assert!(!should_panic);
        },
        counter,
    })
}

fn maybe_error_init(counter: &AtomicUsize, should_error: bool) -> impl Init<CountDrop<'_>, ()> {
    // SAFETY: on `Ok(())` we have written a valid `CountDrop` into `slot`;
    // on `Err(())` we never wrote, so `slot` is left uninitialized as required.
    unsafe {
        init_from_closure(move |slot: *mut CountDrop| {
            if should_error {
                Err(())
            } else {
                slot.write(CountDrop { counter });
                Ok(())
            }
        })
    }
}

fn maybe_error_pin_init(
    counter: &AtomicUsize,
    should_error: bool,
) -> impl PinInit<CountDrop<'_>, ()> {
    // SAFETY: on `Ok(())` we have written a valid `CountDrop` into `slot`;
    // on `Err(())` we never wrote, so `slot` is left uninitialized as required.
    //
    // `CountDrop: Unpin`, so pinning invariants are trivial.
    unsafe {
        pin_init_from_closure(move |slot: *mut CountDrop| {
            if should_error {
                Err(())
            } else {
                slot.write(CountDrop { counter });
                Ok(())
            }
        })
    }
}

#[test]
fn init_array_from_fn_drops_initialized_prefix_on_panic() {
    const N: usize = 10;

    for panic_at in [0, 5, N - 1] {
        let drops = AtomicUsize::new(0);
        let result = std::panic::catch_unwind(|| {
            let _array: [CountDrop<'_>; N] = stack_init(init_array_from_fn(|i| {
                maybe_panicking_init(&drops, i == panic_at)
            }));
        });
        assert!(result.is_err());
        assert_eq!(drops.load(Ordering::Relaxed), panic_at);
    }
}

#[test]
fn pin_init_array_from_fn_drops_initialized_prefix_on_panic() {
    const N: usize = 10;

    for panic_at in [0, 5, N - 1] {
        let drops = AtomicUsize::new(0);
        let result = std::panic::catch_unwind(|| {
            stack_pin_init!(let _array: [CountDrop; N] = pin_init_array_from_fn(|i| {
                maybe_panicking_pin_init(&drops, i == panic_at)
            }));
        });
        assert!(result.is_err());
        assert_eq!(drops.load(Ordering::Relaxed), panic_at);
    }
}

#[test]
fn init_array_from_fn_drops_initialized_prefix_on_error() {
    const N: usize = 10;

    for error_at in [0, 5, N - 1] {
        let drops = AtomicUsize::new(0);
        let result: Result<[CountDrop<'_>; N], ()> = try_stack_init(init_array_from_fn(|i| {
            maybe_error_init(&drops, i == error_at)
        }));
        assert!(result.is_err());
        assert_eq!(drops.load(Ordering::Relaxed), error_at);
    }
}

#[test]
fn pin_init_array_from_fn_drops_initialized_prefix_on_error() {
    const N: usize = 10;

    for error_at in [0, 5, N - 1] {
        let drops = AtomicUsize::new(0);
        stack_try_pin_init!(let result: [CountDrop; N] = pin_init_array_from_fn(|i| {
            maybe_error_pin_init(&drops, i == error_at)
        }));
        assert!(result.is_err());
        assert_eq!(drops.load(Ordering::Relaxed), error_at);
    }
}

#[test]
fn init_array_from_fn_no_double_drop_on_success() {
    const N: usize = 8;

    let drops = AtomicUsize::new(0);
    {
        let _array: [CountDrop<'_>; N] =
            stack_init(init_array_from_fn(|_| maybe_panicking_init(&drops, false)));
        assert_eq!(drops.load(Ordering::Relaxed), 0);
    }
    assert_eq!(drops.load(Ordering::Relaxed), N);
}

#[test]
fn pin_init_array_from_fn_no_double_drop_on_success() {
    const N: usize = 8;

    let drops = AtomicUsize::new(0);
    {
        stack_pin_init!(let _array: [CountDrop; N] =
            pin_init_array_from_fn(|_| maybe_panicking_pin_init(&drops, false))
        );
        assert_eq!(drops.load(Ordering::Relaxed), 0);
    }
    assert_eq!(drops.load(Ordering::Relaxed), N);
}

#[test]
fn chain_init_drops_on_panic() {
    let drops = AtomicUsize::new(0);
    let result = std::panic::catch_unwind(|| {
        let _count_drop: CountDrop<'_> =
            stack_init(maybe_panicking_init(&drops, false).chain(|_| panic!()));
    });
    assert!(result.is_err());
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}

#[test]
fn chain_pin_init_drops_on_panic() {
    let drops = AtomicUsize::new(0);
    let result = std::panic::catch_unwind(|| {
        stack_pin_init!(let _count_drop: CountDrop =
            maybe_panicking_pin_init(&drops, false).pin_chain(|_| panic!())
        );
    });
    assert!(result.is_err());
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}

#[test]
fn chain_init_drops_on_error() {
    let drops = AtomicUsize::new(0);
    let result: Result<CountDrop<'_>, ()> =
        try_stack_init(maybe_error_init(&drops, false).chain(|_| Err(())));
    assert!(result.is_err());
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}

#[test]
fn chain_pin_init_drops_on_error() {
    let drops = AtomicUsize::new(0);
    stack_try_pin_init!(let result: CountDrop =
        maybe_error_init(&drops, false).pin_chain(|_| Err(()))
    );
    assert!(result.is_err());
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}

#[test]
fn chain_init_no_double_drop_on_success() {
    let drops = AtomicUsize::new(0);
    {
        let _count_drop: CountDrop<'_> =
            stack_init(maybe_panicking_init(&drops, false).chain(|_| Ok(())));
        assert_eq!(drops.load(Ordering::Relaxed), 0);
    }
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}

#[test]
fn chain_pin_init_no_double_drop_on_success() {
    let drops = AtomicUsize::new(0);
    {
        stack_pin_init!(let _count_drop: CountDrop =
            maybe_panicking_init(&drops, false).pin_chain(|_| Ok(()))
        );
        assert_eq!(drops.load(Ordering::Relaxed), 0);
    }
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}
