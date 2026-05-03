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

/// Token type to signify successful initialization.
///
/// Can only be constructed via the unsafe [`Self::new`] function. The initializer macros use this
/// token type to prevent returning `Ok` from an initializer without initializing all fields.
pub struct InitOk(());

impl InitOk {
    /// Creates a new token.
    ///
    /// # Safety
    ///
    /// This function may only be called from the `init!` macro in `../internal/src/init.rs`.
    #[inline(always)]
    pub unsafe fn new() -> Self {
        Self(())
    }
}

/// This trait is only implemented via the `#[pin_data]` proc-macro. It is used to facilitate
/// the pin projections within the initializers.
///
/// # Safety
///
/// Only the `init` module is allowed to use this trait.
pub unsafe trait HasPinData {
    type PinData: PinData;

    #[expect(clippy::missing_safety_doc)]
    unsafe fn __pin_data() -> Self::PinData;
}

/// Marker trait for pinning data of structs.
///
/// # Safety
///
/// Only the `init` module is allowed to use this trait.
pub unsafe trait PinData: Copy {
    type Datee: ?Sized + HasPinData;

    /// Type inference helper function.
    #[inline(always)]
    fn make_closure<F, E>(self, f: F) -> F
    where
        F: FnOnce(*mut Self::Datee) -> Result<InitOk, E>,
    {
        f
    }
}

/// This trait is automatically implemented for every type. It aims to provide the same type
/// inference help as `HasPinData`.
///
/// # Safety
///
/// Only the `init` module is allowed to use this trait.
pub unsafe trait HasInitData {
    type InitData: InitData;

    #[expect(clippy::missing_safety_doc)]
    unsafe fn __init_data() -> Self::InitData;
}

/// Same function as `PinData`, but for arbitrary data.
///
/// # Safety
///
/// Only the `init` module is allowed to use this trait.
pub unsafe trait InitData: Copy {
    type Datee: ?Sized + HasInitData;

    /// Type inference helper function.
    #[inline(always)]
    fn make_closure<F, E>(self, f: F) -> F
    where
        F: FnOnce(*mut Self::Datee) -> Result<InitOk, E>,
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

// SAFETY: TODO.
unsafe impl<T: ?Sized> InitData for AllData<T> {
    type Datee = T;
}

// SAFETY: TODO.
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
///
/// # Invariants
///
/// - `ptr` is valid and properly aligned.
/// - `*ptr` is initialized and owned by this guard.
pub struct DropGuard<T: ?Sized> {
    ptr: *mut T,
}

impl<T: ?Sized> DropGuard<T> {
    /// Creates a drop guard and transfer the ownership of the pointer content.
    ///
    /// The ownership is only relinguished if the guard is forgotten via [`core::mem::forget`].
    ///
    /// # Safety
    ///
    /// - `ptr` is valid and properly aligned.
    /// - `*ptr` is initialized, and the ownership is transferred to this guard.
    #[inline]
    pub unsafe fn new(ptr: *mut T) -> Self {
        // INVARIANT: By safety requirement.
        Self { ptr }
    }

    /// Create a let binding for accessor use.
    #[inline]
    pub fn let_binding(&mut self) -> &mut T {
        // SAFETY: Per type invariant.
        unsafe { &mut *self.ptr }
    }
}

impl<T: ?Sized> Drop for DropGuard<T> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: `self.ptr` is valid, properly aligned and `*self.ptr` is owned by this guard.
        unsafe { ptr::drop_in_place(self.ptr) }
    }
}

/// Allows safe (pinned and non-pinned) initialization of an array.
///
/// Drops the already initialized elements of the array if an error or panic occurs
/// partway through the initialization process.
pub struct ArrayInit<T, F> {
    /// A pointer to the first element of the array. Null until `__init` or `__pinned_init`
    /// is called.
    ptr: *mut T,
    /// The number of initialized elements in the array.
    num_init: usize,
    /// Initialization function factory.
    make_init: F,
}

impl<T, F> ArrayInit<T, F> {
    /// # Safety
    ///
    /// This function may only be called from
    /// [`init_array_from_fn`](crate::init_array_from_fn) or
    /// [`pin_init_array_from_fn`](crate::pin_init_array_from_fn).
    pub(crate) unsafe fn new(make_init: F) -> Self {
        Self {
            ptr: core::ptr::null_mut(),
            num_init: 0,
            make_init,
        }
    }
}

/// SAFETY: On success, all `N` elements of the array have been initialized through
/// `I: Init`. On error or panic, the elements that have been initialized so far are
/// dropped, thus leaving the array uninitialized and ready to deallocate. The `Init`
/// implementation executes the same code as that of `PinInit`.
unsafe impl<T, F, I, E, const N: usize> Init<[T; N], E> for ArrayInit<T, F>
where
    F: FnMut(usize) -> I,
    I: Init<T, E>,
{
    unsafe fn __init(mut self, slot: *mut [T; N]) -> Result<(), E> {
        self.ptr = slot.cast::<T>();
        for i in 0..N {
            let init = (self.make_init)(i);
            // SAFETY: Since `0 <= i < N`, `self.ptr.add(i)` is in bounds and
            // valid for writes by the safety contract of `__init`.
            let ptr = unsafe { self.ptr.add(i) };
            // SAFETY: The pointer is derived from `slot` and thus satisfies the
            // `__init` requirements.
            unsafe { init.__init(ptr) }?;
            self.num_init += 1;
        }
        core::mem::forget(self);
        Ok(())
    }
}

/// SAFETY: On success, all `N` elements of the array have been initialized through
/// `I`. Since `I: PinInit` guarantees that the pinning invariants of `T` are upheld,
/// the guarantees of `[T; N]` are also upheld. On error or panic, the elements that
/// have been initialized so far are dropped, thus leaving the array uninitialized
/// and ready to deallocate.
unsafe impl<T, F, I, E, const N: usize> PinInit<[T; N], E> for ArrayInit<T, F>
where
    F: FnMut(usize) -> I,
    I: PinInit<T, E>,
{
    unsafe fn __pinned_init(mut self, slot: *mut [T; N]) -> Result<(), E> {
        self.ptr = slot.cast::<T>();
        for i in 0..N {
            let init = (self.make_init)(i);
            // SAFETY: Since `0 <= i < N`, `self.ptr.add(i)` is in bounds and
            // valid for writes by the safety contract of `__pinned_init`.
            let ptr = unsafe { self.ptr.add(i) };
            // SAFETY: The pointer is derived from `slot` and thus satisfies the
            // `__pinned_init` requirements.
            unsafe { init.__pinned_init(ptr) }?;
            self.num_init += 1;
        }
        core::mem::forget(self);
        Ok(())
    }
}

impl<T, F> Drop for ArrayInit<T, F> {
    fn drop(&mut self) {
        if self.ptr.is_null() {
            // No initialization had been attempted, nothing to drop.
            return;
        }

        // SAFETY: Safety contract of `ArrayInit` guarantees that elements
        // `self.ptr[0..self.num_init]` are initialized and contain valid `T`
        // values, so dropping them is safe.
        unsafe {
            let slice = core::ptr::slice_from_raw_parts_mut(self.ptr, self.num_init);
            core::ptr::drop_in_place(slice)
        };
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
