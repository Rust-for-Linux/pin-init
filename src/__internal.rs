// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This module contains library internal items.
//!
//! These items must not be used outside of this crate and the pin-init-internal crate located at
//! `../internal`.

use super::*;

/// See the [nomicon] for what subtyping is. See also [this table].
///
/// The reason for not using `PhantomData<*mut T>` is that that type never implements [`Send`] and
/// [`Sync`]. Hence `fn(*mut T) -> *mut T` is used, as that type always implements them.
///
/// [nomicon]: https://doc.rust-lang.org/nomicon/subtyping.html
/// [this table]: https://doc.rust-lang.org/nomicon/phantom-data.html#table-of-phantomdata-patterns
pub(crate) type Invariant<T> = PhantomData<fn(*mut T) -> *mut T>;

/// Module-internal type implementing `PinInit` and `Init`.
///
/// It is unsafe to create this type, since the closure needs to fulfill the same safety
/// requirement as the `__pinned_init`/`__init` functions.
pub(crate) struct InitClosure<F, T: ?Sized, E>(pub(crate) F, pub(crate) Invariant<(E, T)>);

// SAFETY: While constructing the `InitClosure`, the user promised that it upholds the
// `__init` invariants.
unsafe impl<T: ?Sized, F, E> Init<T, E> for InitClosure<F, T, E>
where
    F: FnOnce(*mut T) -> Result<(), E>,
{
    #[inline]
    unsafe fn __init(self, slot: *mut T) -> Result<(), E> {
        (self.0)(slot)
    }
}

// SAFETY: While constructing the `InitClosure`, the user promised that it upholds the
// `__pinned_init` invariants.
unsafe impl<T: ?Sized, F, E> PinInit<T, E> for InitClosure<F, T, E>
where
    F: FnOnce(*mut T) -> Result<(), E>,
{
    #[inline]
    unsafe fn __pinned_init(self, slot: *mut T) -> Result<(), E> {
        (self.0)(slot)
    }
}

/// This trait is only implemented via the `#[pin_data]` proc-macro. It is used to facilitate
/// the pin projections within the initializers.
///
/// # Safety
///
/// This `unsafe trait` should only be implemented by the `#[pin_data]` macro. The implementer must ensure:
/// - `Self::PinData` is the correct metadata type for `Self` and implements `PinData<Datee = Self>`
/// - `__pin_data()` returns a valid instance that accurately reflects the structural pinning and field layout of `Self`
/// - The metadata provides sound field accessors that uphold their safety contracts
/// - Incorrect metadata can lead to undefined behavior when used by the pin-init system
pub unsafe trait HasPinData {
    type PinData: PinData;

    /// # Safety
    ///
    /// This method should only be called by macros in the pin-init crate.
    unsafe fn __pin_data() -> Self::PinData;
}

/// Marker trait for pinning data of structs.
///
/// # Safety
///
/// This `unsafe trait` should only be implemented by the `#[pin_data]` macro for its generated
/// metadata struct. The implementer must ensure:
/// - `Self` is `Copy`
/// - `Self::Datee` is the correct struct type implementing `HasPinData<PinData = Self>`
/// - Generated field accessors uphold their safety contracts and correctly use the appropriate
///   initialization methods
pub unsafe trait PinData: Copy {
    type Datee: ?Sized + HasPinData;

    /// Type inference helper function.
    fn make_closure<F, O, E>(self, f: F) -> F
    where
        F: FnOnce(*mut Self::Datee) -> Result<O, E>,
    {
        f
    }
}

/// This trait is automatically implemented for every type. It aims to provide the same type
/// inference help as `HasPinData`.
///
/// # Safety
///
/// This `unsafe trait` should only be implemented by the `pin-init` macro system. The implementer must ensure:
/// - `Self::InitData` accurately represents the initialization structure of `Self`
/// - `__init_data()` returns a valid metadata instance for this type
/// - The metadata correctly reflects the field layout and initialization requirements
pub unsafe trait HasInitData {
    type InitData: InitData;

    /// # Safety
    ///
    /// This method should only be called by macros in the pin-init crate.
    unsafe fn __init_data() -> Self::InitData;
}

/// Same function as `PinData`, but for arbitrary data.
///
/// # Safety
///
/// This `unsafe trait` should only be implemented by the `pin-init` macro system. The implementer must ensure:
/// - `Self` is `Copy`
/// - `Self::Datee` correctly corresponds to the type `Self` represents for initialization purposes
/// - The trait is used consistently within the pin-init system
pub unsafe trait InitData: Copy {
    type Datee: ?Sized + HasInitData;

    /// Type inference helper function.
    fn make_closure<F, O, E>(self, f: F) -> F
    where
        F: FnOnce(*mut Self::Datee) -> Result<O, E>,
    {
        f
    }
}

pub struct AllData<T: ?Sized>(Invariant<T>);

impl<T: ?Sized> Clone for AllData<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for AllData<T> {}

// SAFETY: This implementation upholds the `InitData` invariants that `AllData<T>` is `Copy`, and
// `Self::Datee` is `T`, which correctly represents the type `AllData<T>` is concerned with
// for initialization, as used by the `pin_init!` macros. The `make_closure` method is inherited
// and is a safe identity function, fulfilling trait expectations.
unsafe impl<T: ?Sized> InitData for AllData<T> {
    type Datee = T;
}

// SAFETY: `__init_data` returns `AllData<T>` which is a correct `InitData` implementation
// for type `T`. The function itself performs no unsafe memory operations.
unsafe impl<T: ?Sized> HasInitData for T {
    type InitData = AllData<T>;

    unsafe fn __init_data() -> Self::InitData {
        AllData(PhantomData)
    }
}

/// Stack initializer helper type. Use [`stack_pin_init`] instead of this primitive.
///
/// # Invariants
///
/// If `self.is_init` is true, then `self.value` is initialized.
///
/// [`stack_pin_init`]: crate::stack_pin_init
pub struct StackInit<T> {
    value: MaybeUninit<T>,
    is_init: bool,
}

impl<T> Drop for StackInit<T> {
    #[inline]
    fn drop(&mut self) {
        if self.is_init {
            // SAFETY: As we are being dropped, we only call this once. And since `self.is_init` is
            // true, `self.value` is initialized.
            unsafe { self.value.assume_init_drop() };
        }
    }
}

impl<T> StackInit<T> {
    /// Creates a new [`StackInit<T>`] that is uninitialized. Use [`stack_pin_init`] instead of this
    /// primitive.
    ///
    /// [`stack_pin_init`]: crate::stack_pin_init
    #[inline]
    pub fn uninit() -> Self {
        Self {
            value: MaybeUninit::uninit(),
            is_init: false,
        }
    }

    /// Initializes the contents and returns the result.
    #[inline]
    pub fn init<E>(self: Pin<&mut Self>, init: impl PinInit<T, E>) -> Result<Pin<&mut T>, E> {
        // SAFETY: We never move out of `this`.
        let this = unsafe { Pin::into_inner_unchecked(self) };
        // The value is currently initialized, so it needs to be dropped before we can reuse
        // the memory (this is a safety guarantee of `Pin`).
        if this.is_init {
            this.is_init = false;
            // SAFETY: `this.is_init` was true and therefore `this.value` is initialized.
            unsafe { this.value.assume_init_drop() };
        }
        // SAFETY: The memory slot is valid and this type ensures that it will stay pinned.
        unsafe { init.__pinned_init(this.value.as_mut_ptr())? };
        // INVARIANT: `this.value` is initialized above.
        this.is_init = true;
        // SAFETY: The slot is now pinned, since we will never give access to `&mut T`.
        Ok(unsafe { Pin::new_unchecked(this.value.assume_init_mut()) })
    }
}

#[test]
#[cfg(feature = "std")]
fn stack_init_reuse() {
    use ::std::{borrow::ToOwned, println, string::String};
    use core::pin::pin;

    #[derive(Debug)]
    struct Foo {
        a: usize,
        b: String,
    }
    let mut slot: Pin<&mut StackInit<Foo>> = pin!(StackInit::uninit());
    let value: Result<Pin<&mut Foo>, core::convert::Infallible> =
        slot.as_mut().init(crate::init!(Foo {
            a: 42,
            b: "Hello".to_owned(),
        }));
    let value = value.unwrap();
    println!("{value:?}");
    let value: Result<Pin<&mut Foo>, core::convert::Infallible> =
        slot.as_mut().init(crate::init!(Foo {
            a: 24,
            b: "world!".to_owned(),
        }));
    let value = value.unwrap();
    println!("{value:?}");
}

/// When a value of this type is dropped, it drops a `T`.
///
/// Can be forgotten to prevent the drop.
pub struct DropGuard<T: ?Sized> {
    ptr: *mut T,
}

impl<T: ?Sized> DropGuard<T> {
    /// Creates a new [`DropGuard<T>`]. It will [`ptr::drop_in_place`] `ptr` when it gets dropped.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid pointer.
    ///
    /// It is the callers responsibility that `self` will only get dropped if the pointee of `ptr`:
    /// - has not been dropped,
    /// - is not accessible by any other means,
    /// - will not be dropped by any other means.
    #[inline]
    pub unsafe fn new(ptr: *mut T) -> Self {
        Self { ptr }
    }
}

impl<T: ?Sized> Drop for DropGuard<T> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: A `DropGuard` can only be constructed using the unsafe `new` function
        // ensuring that this operation is safe.
        unsafe { ptr::drop_in_place(self.ptr) }
    }
}

/// Token used by `PinnedDrop` to prevent calling the function without creating this unsafely
/// created struct. This is needed, because the `drop` function is safe, but should not be called
/// manually.
pub struct OnlyCallFromDrop(());

impl OnlyCallFromDrop {
    /// # Safety
    ///
    /// This function should only be called from the [`Drop::drop`] function and only be used to
    /// delegate the destruction to the pinned destructor [`PinnedDrop::drop`] of the same type.
    pub unsafe fn new() -> Self {
        Self(())
    }
}

/// Initializer that always fails.
///
/// Used by [`assert_pinned!`].
///
/// [`assert_pinned!`]: crate::assert_pinned
pub struct AlwaysFail<T: ?Sized> {
    _t: PhantomData<T>,
}

impl<T: ?Sized> AlwaysFail<T> {
    /// Creates a new initializer that always fails.
    pub fn new() -> Self {
        Self { _t: PhantomData }
    }
}

impl<T: ?Sized> Default for AlwaysFail<T> {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: `__pinned_init` always fails, which is always okay.
unsafe impl<T: ?Sized> PinInit<T, ()> for AlwaysFail<T> {
    unsafe fn __pinned_init(self, _slot: *mut T) -> Result<(), ()> {
        Err(())
    }
}
