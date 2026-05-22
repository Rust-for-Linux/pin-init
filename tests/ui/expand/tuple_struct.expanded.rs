use core::marker::PhantomPinned;
use pin_init::*;
struct Foo<'a, T: Copy, const N: usize>(&'a mut [T; N], PhantomPinned, usize);
/// Pin-projections of [`Foo`]
#[allow(dead_code)]
#[doc(hidden)]
struct FooProjection<'__pin, 'a, T: Copy, const N: usize>(
    &'__pin mut &'a mut [T; N],
    ::core::pin::Pin<&'__pin mut PhantomPinned>,
    &'__pin mut usize,
    ::core::marker::PhantomData<&'__pin mut ()>,
);
impl<'a, T: Copy, const N: usize> Foo<'a, T, N> {
    /// Pin-projects all fields of `Self`.
    ///
    /// These fields are structurally pinned:
    /// - index `1`
    ///
    /// These fields are **not** structurally pinned:
    /// - index `0`
    /// - index `2`
    #[inline]
    fn project<'__pin>(
        self: ::core::pin::Pin<&'__pin mut Self>,
    ) -> FooProjection<'__pin, 'a, T, N> {
        let this = unsafe { ::core::pin::Pin::get_unchecked_mut(self) };
        {
            FooProjection(
                &mut this.0,
                unsafe { ::core::pin::Pin::new_unchecked(&mut this.1) },
                &mut this.2,
                ::core::marker::PhantomData,
            )
        }
    }
}
const _: () = {
    #[doc(hidden)]
    struct __ThePinData<'a, T: Copy, const N: usize> {
        __phantom: ::pin_init::__internal::PhantomInvariant<Foo<'a, T, N>>,
    }
    impl<'a, T: Copy, const N: usize> ::core::clone::Clone for __ThePinData<'a, T, N> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<'a, T: Copy, const N: usize> ::core::marker::Copy for __ThePinData<'a, T, N> {}
    #[allow(dead_code)]
    #[expect(clippy::missing_safety_doc)]
    impl<'a, T: Copy, const N: usize> __ThePinData<'a, T, N> {
        /// Type inference helper function.
        #[inline(always)]
        fn __make_closure<__F, __E>(self, f: __F) -> __F
        where
            __F: FnOnce(
                *mut Foo<'a, T, N>,
            ) -> ::core::result::Result<::pin_init::__internal::InitOk, __E>,
        {
            f
        }
        /// # Safety
        ///
        /// - `slot` is valid and properly aligned.
        /// - The field is properly aligned.
        /// - The field points to uninitialized and exclusively accessed memory.
        #[inline(always)]
        unsafe fn _0(
            self,
            slot: *mut Foo<'a, T, N>,
        ) -> ::pin_init::__internal::Slot<
            ::pin_init::__internal::Unpinned,
            &'a mut [T; N],
        > {
            unsafe { ::pin_init::__internal::Slot::new(&raw mut (*slot).0) }
        }
        /// # Safety
        ///
        /// - `slot` is valid and properly aligned.
        /// - The field is properly aligned.
        /// - The field points to uninitialized and exclusively accessed memory.
        #[inline(always)]
        unsafe fn _1(
            self,
            slot: *mut Foo<'a, T, N>,
        ) -> ::pin_init::__internal::Slot<
            ::pin_init::__internal::Pinned,
            PhantomPinned,
        > {
            unsafe { ::pin_init::__internal::Slot::new(&raw mut (*slot).1) }
        }
        /// # Safety
        ///
        /// - `slot` is valid and properly aligned.
        /// - The field is properly aligned.
        /// - The field points to uninitialized and exclusively accessed memory.
        #[inline(always)]
        unsafe fn _2(
            self,
            slot: *mut Foo<'a, T, N>,
        ) -> ::pin_init::__internal::Slot<::pin_init::__internal::Unpinned, usize> {
            unsafe { ::pin_init::__internal::Slot::new(&raw mut (*slot).2) }
        }
    }
    unsafe impl<'a, T: Copy, const N: usize> ::pin_init::__internal::HasPinData
    for Foo<'a, T, N> {
        type PinData = __ThePinData<'a, T, N>;
        unsafe fn __pin_data() -> Self::PinData {
            __ThePinData {
                __phantom: ::pin_init::__internal::PhantomInvariant::new(),
            }
        }
    }
    #[allow(dead_code)]
    struct __Unpin<'__pin, 'a, T: Copy, const N: usize> {
        __phantom_pin: ::pin_init::__internal::PhantomInvariantLifetime<'__pin>,
        __phantom: ::pin_init::__internal::PhantomInvariant<Foo<'a, T, N>>,
        _1: PhantomPinned,
    }
    #[doc(hidden)]
    impl<'__pin, 'a, T: Copy, const N: usize> ::core::marker::Unpin for Foo<'a, T, N>
    where
        __Unpin<'__pin, 'a, T, N>: ::core::marker::Unpin,
    {}
    trait MustNotImplDrop {}
    #[expect(drop_bounds)]
    impl<T: ::core::ops::Drop + ?::core::marker::Sized> MustNotImplDrop for T {}
    impl<'a, T: Copy, const N: usize> MustNotImplDrop for Foo<'a, T, N> {}
    #[expect(non_camel_case_types)]
    trait UselessPinnedDropImpl_you_need_to_specify_PinnedDrop {}
    impl<
        T: ::pin_init::PinnedDrop + ?::core::marker::Sized,
    > UselessPinnedDropImpl_you_need_to_specify_PinnedDrop for T {}
    impl<
        'a,
        T: Copy,
        const N: usize,
    > UselessPinnedDropImpl_you_need_to_specify_PinnedDrop for Foo<'a, T, N> {}
};
fn main() {}
