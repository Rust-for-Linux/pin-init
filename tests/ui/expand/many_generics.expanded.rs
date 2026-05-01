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
const _: () = {
    use Foo as __DropCheck;
    /// Pin-projections of [`Foo`]
    #[allow(dead_code, non_snake_case)]
    #[doc(hidden)]
    struct __Projection<
        '__pin,
        'a,
        'b: 'a,
        T: Bar<'b> + ?Sized + 'a,
        const SIZE: usize = 0,
    >
    where
        T: Bar<'a, 1>,
    {
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
        ) -> __Projection<'__pin, 'a, 'b, T, SIZE> {
            let this = unsafe { ::core::pin::Pin::get_unchecked_mut(self) };
            __Projection {
                array: &mut this.array,
                r: &mut this.r,
                _pin: unsafe { ::core::pin::Pin::new_unchecked(&mut this._pin) },
                ___pin_phantom_data: ::core::marker::PhantomData,
            }
        }
    }
    #[doc(hidden)]
    struct __ThePinData<'a, 'b: 'a, T: Bar<'b> + ?Sized + 'a, const SIZE: usize = 0>
    where
        T: Bar<'a, 1>,
    {
        __phantom: ::pin_init::__internal::PhantomInvariant<Foo<'a, 'b, T, SIZE>>,
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
    impl<
        'a,
        'b: 'a,
        T: Bar<'b> + ?Sized + 'a,
        const SIZE: usize,
    > __ThePinData<'a, 'b, T, SIZE>
    where
        T: Bar<'a, 1>,
    {
        /// Type inference helper function.
        #[inline(always)]
        fn __make_closure<__F, __E>(self, f: __F) -> __F
        where
            __F: FnOnce(
                *mut Foo<'a, 'b, T, SIZE>,
                __ThePinData<'a, 'b, T, SIZE>,
            ) -> ::core::result::Result<::pin_init::__internal::InitOk, __E>,
        {
            f
        }
        /// # Safety
        ///
        /// - `slot` is valid and properly aligned.
        /// - `(*slot).#field_name` is properly aligned.
        /// - `(*slot).#field_name` points to uninitialized and exclusively accessed
        ///   memory.
        #[allow(non_snake_case)]
        #[inline(always)]
        unsafe fn array(
            self,
            slot: *mut Foo<'a, 'b, T, SIZE>,
        ) -> ::pin_init::__internal::Slot<
            ::pin_init::__internal::Unpinned,
            [u8; 1024 * 1024],
        > {
            unsafe { ::pin_init::__internal::Slot::new(&raw mut (*slot).array) }
        }
        /// # Safety
        ///
        /// - `slot` is valid and properly aligned.
        /// - `(*slot).#field_name` is properly aligned.
        /// - `(*slot).#field_name` points to uninitialized and exclusively accessed
        ///   memory.
        #[allow(non_snake_case)]
        #[inline(always)]
        unsafe fn r(
            self,
            slot: *mut Foo<'a, 'b, T, SIZE>,
        ) -> ::pin_init::__internal::Slot<
            ::pin_init::__internal::Unpinned,
            &'b mut [&'a mut T; SIZE],
        > {
            unsafe { ::pin_init::__internal::Slot::new(&raw mut (*slot).r) }
        }
        /// # Safety
        ///
        /// - `slot` is valid and properly aligned.
        /// - `(*slot).#field_name` is properly aligned.
        /// - `(*slot).#field_name` points to uninitialized and exclusively accessed
        ///   memory.
        #[allow(non_snake_case)]
        #[inline(always)]
        unsafe fn _pin(
            self,
            slot: *mut Foo<'a, 'b, T, SIZE>,
        ) -> ::pin_init::__internal::Slot<
            ::pin_init::__internal::Pinned,
            PhantomPinned,
        > {
            unsafe { ::pin_init::__internal::Slot::new(&raw mut (*slot)._pin) }
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
                __phantom: ::pin_init::__internal::PhantomInvariant::new(),
            }
        }
    }
    #[allow(dead_code, non_snake_case)]
    struct __Unpin<'__pin, 'a, 'b: 'a, T: Bar<'b> + ?Sized + 'a, const SIZE: usize = 0>
    where
        T: Bar<'a, 1>,
    {
        __phantom_pin: ::pin_init::__internal::PhantomInvariantLifetime<'__pin>,
        __phantom: ::pin_init::__internal::PhantomInvariant<Foo<'a, 'b, T, SIZE>>,
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
