use pin_init::*;
struct Foo([u8; 1024 * 1024], i32);
/// Pin-projections of [`Foo`]
#[allow(dead_code)]
#[doc(hidden)]
struct FooProjection<'__pin> {
    _0: &'__pin mut [u8; 1024 * 1024],
    _1: ::core::pin::Pin<&'__pin mut i32>,
    ___pin_phantom_data: ::core::marker::PhantomData<&'__pin mut ()>,
}
impl Foo {
    /// Pin-projects all fields of `Self`.
    ///
    /// These fields are structurally pinned:
    /// - index `1`
    ///
    /// These fields are **not** structurally pinned:
    /// - index `0`
    #[inline]
    fn project<'__pin>(
        self: ::core::pin::Pin<&'__pin mut Self>,
    ) -> FooProjection<'__pin> {
        let this = unsafe { ::core::pin::Pin::get_unchecked_mut(self) };
        FooProjection {
            _0: &mut this.0,
            _1: unsafe { ::core::pin::Pin::new_unchecked(&mut this.1) },
            ___pin_phantom_data: ::core::marker::PhantomData,
        }
    }
}
const _: () = {
    #[doc(hidden)]
    struct __ThePinData {
        __phantom: ::core::marker::PhantomData<fn(Foo) -> Foo>,
    }
    impl ::core::clone::Clone for __ThePinData {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl ::core::marker::Copy for __ThePinData {}
    #[allow(dead_code)]
    #[expect(clippy::missing_safety_doc)]
    impl __ThePinData {
        /// # Safety
        ///
        /// - `slot` is a valid pointer to uninitialized memory.
        /// - the caller does not touch `slot` when `Err` is returned, they are only permitted
        ///   to deallocate.
        unsafe fn _0<E>(
            self,
            slot: *mut [u8; 1024 * 1024],
            init: impl ::pin_init::Init<[u8; 1024 * 1024], E>,
        ) -> ::core::result::Result<(), E> {
            unsafe { ::pin_init::Init::__init(init, slot) }
        }
        /// # Safety
        ///
        /// `slot` points at the field index `0` inside of `Foo`, which is pinned.
        unsafe fn __project_0<'__slot>(
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
        /// - `slot` will not move until it is dropped, i.e. it will be pinned.
        unsafe fn _1<E>(
            self,
            slot: *mut i32,
            init: impl ::pin_init::PinInit<i32, E>,
        ) -> ::core::result::Result<(), E> {
            unsafe { ::pin_init::PinInit::__pinned_init(init, slot) }
        }
        /// # Safety
        ///
        /// `slot` points at the field index `1` inside of `Foo`, which is pinned.
        unsafe fn __project_1<'__slot>(
            self,
            slot: &'__slot mut i32,
        ) -> ::core::pin::Pin<&'__slot mut i32> {
            unsafe { ::core::pin::Pin::new_unchecked(slot) }
        }
    }
    unsafe impl ::pin_init::__internal::HasPinData for Foo {
        type PinData = __ThePinData;
        unsafe fn __pin_data() -> Self::PinData {
            __ThePinData {
                __phantom: ::core::marker::PhantomData,
            }
        }
    }
    unsafe impl ::pin_init::__internal::PinData for __ThePinData {
        type Datee = Foo;
    }
    #[allow(dead_code)]
    struct __Unpin<'__pin> {
        __phantom_pin: ::core::marker::PhantomData<fn(&'__pin ()) -> &'__pin ()>,
        __phantom: ::core::marker::PhantomData<fn(Foo) -> Foo>,
        _1: i32,
    }
    #[doc(hidden)]
    impl<'__pin> ::core::marker::Unpin for Foo
    where
        __Unpin<'__pin>: ::core::marker::Unpin,
    {}
    trait MustNotImplDrop {}
    #[expect(drop_bounds)]
    impl<T: ::core::ops::Drop + ?::core::marker::Sized> MustNotImplDrop for T {}
    impl MustNotImplDrop for Foo {}
    #[expect(non_camel_case_types)]
    trait UselessPinnedDropImpl_you_need_to_specify_PinnedDrop {}
    impl<
        T: ::pin_init::PinnedDrop + ?::core::marker::Sized,
    > UselessPinnedDropImpl_you_need_to_specify_PinnedDrop for T {}
    impl UselessPinnedDropImpl_you_need_to_specify_PinnedDrop for Foo {}
};
