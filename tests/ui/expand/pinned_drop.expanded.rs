use core::{marker::PhantomPinned, pin::Pin};
use pin_init::*;
struct Foo {
    array: [u8; 1024 * 1024],
    _pin: PhantomPinned,
}
const _: () = {
    /// Pin-projections of [`Foo`]
    #[allow(dead_code, non_snake_case)]
    #[doc(hidden)]
    struct __Projection<'__this> {
        array: &'__this mut [u8; 1024 * 1024],
        _pin: ::core::pin::Pin<&'__this mut PhantomPinned>,
        __this: ::core::marker::PhantomData<&'__this mut Foo>,
    }
    /// Pin-projections of [`Foo`]
    #[allow(dead_code, non_snake_case)]
    #[doc(hidden)]
    struct __ProjectionLt<'__this> {
        array: &'__this mut [u8; 1024 * 1024],
        _pin: ::core::pin::Pin<&'__this mut PhantomPinned>,
        ___pin_phantom_data: ::core::marker::PhantomData<&'__this mut Foo>,
    }
    impl Foo {
        /// Pin-projects all fields of `Self`.
        ///
        /// These fields are structurally pinned:
        /// - `_pin`
        ///
        /// These fields are **not** structurally pinned:
        /// - `array`
        #[inline]
        fn project<'__this>(
            self: ::core::pin::Pin<&'__this mut Self>,
        ) -> __Projection<'__this> {
            let this = unsafe { ::core::pin::Pin::get_unchecked_mut(self) };
            __Projection {
                array: &mut this.array,
                _pin: unsafe { ::core::pin::Pin::new_unchecked(&mut this._pin) },
                __this: ::core::marker::PhantomData,
            }
        }
        /// Pin-projects all fields of `Self` with proper lifetime.
        ///
        /// These fields are structurally pinned:
        /// - `_pin`
        ///
        /// These fields are **not** structurally pinned:
        /// - `array`
        #[inline]
        fn with_project<'__this, R>(
            self: ::core::pin::Pin<&'__this mut Self>,
            f: impl ::core::ops::FnOnce(__ProjectionLt<'__this>) -> R,
        ) -> R {
            let this = unsafe { ::core::pin::Pin::get_unchecked_mut(self) };
            f(__ProjectionLt {
                array: &mut this.array,
                _pin: unsafe { ::core::pin::Pin::new_unchecked(&mut this._pin) },
                ___pin_phantom_data: ::core::marker::PhantomData,
            })
        }
    }
    #[doc(hidden)]
    #[allow(non_snake_case)]
    struct __PinDataLt {
        array: ::pin_init::__internal::PhantomInvariant<[u8; 1024 * 1024]>,
        _pin: ::pin_init::__internal::PhantomInvariant<PhantomPinned>,
        __pin_phantom: ::core::marker::PhantomData<Foo>,
    }
    impl ::core::clone::Clone for __PinDataLt {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl ::core::marker::Copy for __PinDataLt {}
    #[allow(dead_code)]
    #[expect(clippy::missing_safety_doc)]
    impl __PinDataLt {
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
            slot: *mut Foo,
        ) -> ::pin_init::__internal::Slot<
            ::pin_init::__internal::Unpinned,
            [u8; 1024 * 1024],
        > {
            unsafe { ::pin_init::__internal::Slot::new(&raw mut (*slot).array as _) }
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
            slot: *mut Foo,
        ) -> ::pin_init::__internal::Slot<
            ::pin_init::__internal::Pinned,
            PhantomPinned,
        > {
            unsafe { ::pin_init::__internal::Slot::new(&raw mut (*slot)._pin as _) }
        }
    }
    #[doc(hidden)]
    struct __ThePinData {
        __phantom: ::pin_init::__internal::PhantomInvariant<Foo>,
    }
    impl ::core::clone::Clone for __ThePinData {
        #[inline]
        fn clone(&self) -> Self {
            *self
        }
    }
    impl ::core::marker::Copy for __ThePinData {}
    impl __ThePinData {
        /// Type inference helper function.
        #[inline(always)]
        fn __make_closure<__F, __E>(self, f: __F) -> __F
        where
            __F: ::core::ops::FnOnce(
                *mut Foo,
                __PinDataLt,
            ) -> ::core::result::Result<::pin_init::__internal::InitOk, __E>,
        {
            f
        }
        #[inline(always)]
        fn __with_lt(self) -> __PinDataLt {
            unsafe { ::core::mem::zeroed() }
        }
    }
    unsafe impl ::pin_init::__internal::HasPinData for Foo {
        type PinData = __ThePinData;
        #[inline]
        fn __pin_data(_: ::pin_init::__internal::InitData<Self>) -> Self::PinData {
            __ThePinData {
                __phantom: ::pin_init::__internal::PhantomInvariant::new(),
            }
        }
    }
    #[allow(dead_code, non_snake_case)]
    struct __Unpin {
        __phantom: ::pin_init::__internal::PhantomInvariant<Foo>,
        _pin: PhantomPinned,
    }
    #[doc(hidden)]
    impl ::core::marker::Unpin for Foo
    where
        for<'__dummy> __Unpin: ::core::marker::Unpin,
    {}
    impl ::core::ops::Drop for Foo {
        fn drop(&mut self) {
            let pinned = unsafe { ::core::pin::Pin::new_unchecked(self) };
            let token = unsafe { ::pin_init::__internal::OnlyCallFromDrop::new() };
            ::pin_init::PinnedDrop::drop(pinned, token);
        }
    }
};
unsafe impl ::pin_init::PinnedDrop for Foo {
    fn drop(self: Pin<&mut Self>, _: ::pin_init::__internal::OnlyCallFromDrop) {}
}
