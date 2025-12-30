#![feature(lint_reasons)]
#![allow(dead_code)]
use core::{marker::PhantomPinned, pin::Pin};
use pin_init::*;
trait Bar<'a, const ID: usize = 0> {
    fn bar(&mut self);
}
struct Foo<'a, 'b: 'a, T: Bar<'b> + ?Sized + 'a, const SIZE: usize = 0>
where
    T: Bar<'a, 1>,
{
    array: [u8; 1024 * 1024],
    r: &'b mut [&'a mut T; SIZE],
    _pin: PhantomPinned,
}
/// Pin-projections of [`Foo`]
#[allow(dead_code)]
#[doc(hidden)]
struct FooProjection<
    '__pin,
    'a,
    'b: 'a,
    T: Bar<'b> + ?Sized + 'a,
    const SIZE: usize = 0,
> {
    array: &'__pin mut [u8; 1024 * 1024],
    r: &'__pin mut &'b mut [&'a mut T; SIZE],
    _pin: ::core::pin::Pin<&'__pin mut PhantomPinned>,
    ___pin_phantom_data: ::core::marker::PhantomData<&'__pin mut ()>,
}
impl<'a, 'b: 'a, T: Bar<'b> + ?Sized + 'a, const SIZE: usize> Foo<'a, 'b, T, SIZE>
where
    T: Bar<'a, 1>,
{
    /// Pin-projects all fields of `Self`.
    ///
    /// These fields are structurally pinned:
    /// - `_pin`
    ///
    /// These fields are **not** structurally pinned:
    /// - `array`
    /// - `r`
    #[inline]
    fn project<'__pin>(
        self: ::core::pin::Pin<&'__pin mut Self>,
    ) -> FooProjection<'__pin, 'a, 'b, T, SIZE> {
        let this = unsafe { ::core::pin::Pin::get_unchecked_mut(self) };
        FooProjection {
            array: &mut this.array,
            r: &mut this.r,
            _pin: unsafe { ::core::pin::Pin::new_unchecked(&mut this._pin) },
            ___pin_phantom_data: ::core::marker::PhantomData,
        }
    }
}
const _: () = {
    #[doc(hidden)]
    struct __ThePinData<'a, 'b: 'a, T: Bar<'b> + ?Sized + 'a, const SIZE: usize = 0>
    where
        T: Bar<'a, 1>,
    {
        __phantom: ::core::marker::PhantomData<
            fn(Foo<'a, 'b, T, SIZE>) -> Foo<'a, 'b, T, SIZE>,
        >,
    }
    impl<'a, 'b: 'a, T: Bar<'b> + ?Sized + 'a, const SIZE: usize> ::core::clone::Clone
    for __ThePinData<'a, 'b, T, SIZE>
    where
        T: Bar<'a, 1>,
    {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<'a, 'b: 'a, T: Bar<'b> + ?Sized + 'a, const SIZE: usize> ::core::marker::Copy
    for __ThePinData<'a, 'b, T, SIZE>
    where
        T: Bar<'a, 1>,
    {}
    #[allow(dead_code)]
    #[expect(clippy::missing_safety_doc)]
    impl<
        'a,
        'b: 'a,
        T: Bar<'b> + ?Sized + 'a,
        const SIZE: usize,
    > __ThePinData<'a, 'b, T, SIZE>
    where
        T: Bar<'a, 1>,
    {
        /// # Safety
        ///
        /// - `slot` is a valid pointer to uninitialized memory.
        /// - the caller does not touch `slot` when `Err` is returned, they are only permitted
        ///   to deallocate.
        unsafe fn array<E>(
            self,
            slot: *mut [u8; 1024 * 1024],
            init: impl ::pin_init::Init<[u8; 1024 * 1024], E>,
        ) -> ::core::result::Result<(), E> {
            unsafe { ::pin_init::Init::__init(init, slot) }
        }
        /// # Safety
        ///
        /// `slot` points at the field `array` inside of `Foo`, which is pinned.
        unsafe fn __project_array<'__slot>(
            self,
            slot: &'__slot mut [u8; 1024 * 1024],
        ) -> &'__slot mut [u8; 1024 * 1024] {
            slot
        }
        /// # Safety
        ///
        /// - `slot` is a valid pointer to uninitialized memory.
        /// - the caller does not touch `slot` when `Err` is returned, they are only permitted
        ///   to deallocate.
        unsafe fn r<E>(
            self,
            slot: *mut &'b mut [&'a mut T; SIZE],
            init: impl ::pin_init::Init<&'b mut [&'a mut T; SIZE], E>,
        ) -> ::core::result::Result<(), E> {
            unsafe { ::pin_init::Init::__init(init, slot) }
        }
        /// # Safety
        ///
        /// `slot` points at the field `r` inside of `Foo`, which is pinned.
        unsafe fn __project_r<'__slot>(
            self,
            slot: &'__slot mut &'b mut [&'a mut T; SIZE],
        ) -> &'__slot mut &'b mut [&'a mut T; SIZE] {
            slot
        }
        /// # Safety
        ///
        /// - `slot` is a valid pointer to uninitialized memory.
        /// - the caller does not touch `slot` when `Err` is returned, they are only permitted
        ///   to deallocate.
        /// - `slot` will not move until it is dropped, i.e. it will be pinned.
        unsafe fn _pin<E>(
            self,
            slot: *mut PhantomPinned,
            init: impl ::pin_init::PinInit<PhantomPinned, E>,
        ) -> ::core::result::Result<(), E> {
            unsafe { ::pin_init::PinInit::__pinned_init(init, slot) }
        }
        /// # Safety
        ///
        /// `slot` points at the field `_pin` inside of `Foo`, which is pinned.
        unsafe fn __project__pin<'__slot>(
            self,
            slot: &'__slot mut PhantomPinned,
        ) -> ::core::pin::Pin<&'__slot mut PhantomPinned> {
            unsafe { ::core::pin::Pin::new_unchecked(slot) }
        }
    }
    unsafe impl<
        'a,
        'b: 'a,
        T: Bar<'b> + ?Sized + 'a,
        const SIZE: usize,
    > ::pin_init::__internal::HasPinData for Foo<'a, 'b, T, SIZE>
    where
        T: Bar<'a, 1>,
    {
        type PinData = __ThePinData<'a, 'b, T, SIZE>;
        unsafe fn __pin_data() -> Self::PinData {
            __ThePinData {
                __phantom: ::core::marker::PhantomData,
            }
        }
    }
    unsafe impl<
        'a,
        'b: 'a,
        T: Bar<'b> + ?Sized + 'a,
        const SIZE: usize,
    > ::pin_init::__internal::PinData for __ThePinData<'a, 'b, T, SIZE>
    where
        T: Bar<'a, 1>,
    {
        type Datee = Foo<'a, 'b, T, SIZE>;
    }
    #[allow(dead_code)]
    struct __Unpin<'__pin, 'a, 'b: 'a, T: Bar<'b> + ?Sized + 'a, const SIZE: usize = 0>
    where
        T: Bar<'a, 1>,
    {
        __phantom_pin: ::core::marker::PhantomData<fn(&'__pin ()) -> &'__pin ()>,
        __phantom: ::core::marker::PhantomData<
            fn(Foo<'a, 'b, T, SIZE>) -> Foo<'a, 'b, T, SIZE>,
        >,
        _pin: PhantomPinned,
    }
    #[doc(hidden)]
    impl<
        '__pin,
        'a,
        'b: 'a,
        T: Bar<'b> + ?Sized + 'a,
        const SIZE: usize,
    > ::core::marker::Unpin for Foo<'a, 'b, T, SIZE>
    where
        __Unpin<'__pin, 'a, 'b, T, SIZE>: ::core::marker::Unpin,
        T: Bar<'a, 1>,
    {}
    impl<'a, 'b: 'a, T: Bar<'b> + ?Sized + 'a, const SIZE: usize> ::core::ops::Drop
    for Foo<'a, 'b, T, SIZE>
    where
        T: Bar<'a, 1>,
    {
        fn drop(&mut self) {
            let pinned = unsafe { ::core::pin::Pin::new_unchecked(self) };
            let token = unsafe { ::pin_init::__internal::OnlyCallFromDrop::new() };
            ::pin_init::PinnedDrop::drop(pinned, token);
        }
    }
};
unsafe impl<
    'a,
    'b: 'a,
    T: Bar<'b> + ?Sized + 'a,
    const SIZE: usize,
> ::pin_init::PinnedDrop for Foo<'a, 'b, T, SIZE>
where
    T: Bar<'b, 1>,
{
    fn drop(self: Pin<&mut Self>, _: ::pin_init::__internal::OnlyCallFromDrop) {
        let me = unsafe { Pin::get_unchecked_mut(self) };
        for t in &mut *me.r {
            Bar::<'a, 1>::bar(*t);
        }
    }
}
fn main() {}
