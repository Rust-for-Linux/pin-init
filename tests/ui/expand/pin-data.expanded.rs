use core::marker::PhantomPinned;
use pin_init::*;
struct Foo {
    array: [u8; 1024 * 1024],
    _pin: PhantomPinned,
}
/// Pin-projections of [`Foo`]
#[allow(dead_code)]
#[doc(hidden)]
struct FooProjection<'__pin> {
    array: &'__pin mut [u8; 1024 * 1024],
    _pin: ::core::pin::Pin<&'__pin mut PhantomPinned>,
    ___pin_phantom_data: ::core::marker::PhantomData<&'__pin mut ()>,
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
    fn project<'__pin>(
        self: ::core::pin::Pin<&'__pin mut Self>,
    ) -> FooProjection<'__pin> {
        let this = unsafe { ::core::pin::Pin::get_unchecked_mut(self) };
        FooProjection {
            array: &mut this.array,
            _pin: unsafe { ::core::pin::Pin::new_unchecked(&mut this._pin) },
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
        unsafe fn array<'__slot>(
            self,
            slot: *mut [u8; 1024 * 1024],
        ) -> ::pin_init::__internal::UnpinnedSlot<'__slot, [u8; 1024 * 1024]> {
            unsafe { ::pin_init::__internal::UnpinnedSlot::new(slot) }
        }
        /// # Safety
        ///
        /// - `slot` is a valid pointer to uninitialized memory.
        unsafe fn _pin<'__slot>(
            self,
            slot: *mut PhantomPinned,
        ) -> ::pin_init::__internal::PinnedSlot<'__slot, PhantomPinned> {
            unsafe { ::pin_init::__internal::PinnedSlot::new(slot) }
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
    impl __ThePinData {
        /// Type inference helper function.
        #[inline(always)]
        fn __make_init<F, E>(self, f: F) -> impl ::pin_init::PinInit<Foo, E>
        where
            F: ::core::ops::FnOnce(
                *mut Foo,
            ) -> ::core::result::Result<::pin_init::__internal::InitOk, E>,
        {
            unsafe {
                ::pin_init::pin_init_from_closure(move |
                    slot,
                | -> ::core::result::Result<(), E> {
                    f(slot)?;
                    Ok(())
                })
            }
        }
    }
    #[allow(dead_code)]
    struct __Unpin<'__pin> {
        __phantom_pin: ::core::marker::PhantomData<fn(&'__pin ()) -> &'__pin ()>,
        __phantom: ::core::marker::PhantomData<fn(Foo) -> Foo>,
        _pin: PhantomPinned,
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
