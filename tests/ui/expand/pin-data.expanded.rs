use core::marker::PhantomPinned;
use pin_init::*;
struct Foo {
    array: [u8; 1024 * 1024],
    _pin: PhantomPinned,
}
#[doc(hidden)]
#[allow(dead_code)]
struct FooProjection<'__pin> {
    _pin: ::core::pin::Pin<&'__pin mut PhantomPinned>,
    array: &'__pin mut [u8; 1024 * 1024],
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
            _pin: unsafe { ::core::pin::Pin::new_unchecked(&mut this._pin) },
            array: &mut this.array,
            ___pin_phantom_data: ::core::marker::PhantomData,
        }
    }
}
const _: () = {
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
        unsafe fn _pin<E>(
            self,
            slot: *mut PhantomPinned,
            init: impl ::pin_init::PinInit<PhantomPinned, E>,
        ) -> ::core::result::Result<(), E> {
            unsafe { ::pin_init::PinInit::__pinned_init(init, slot) }
        }
        unsafe fn __project__pin<'__slot>(
            self,
            slot: &'__slot mut PhantomPinned,
        ) -> ::core::pin::Pin<&'__slot mut PhantomPinned> {
            ::core::pin::Pin::new_unchecked(slot)
        }
        #[allow(non_snake_case)]
        unsafe fn array<E>(
            self,
            slot: *mut [u8; 1024 * 1024],
            init: impl ::pin_init::Init<[u8; 1024 * 1024], E>,
        ) -> ::core::result::Result<(), E> {
            unsafe { ::pin_init::Init::__init(init, slot) }
        }
        #[allow(non_snake_case)]
        unsafe fn __project_array<'__slot>(
            self,
            slot: &'__slot mut [u8; 1024 * 1024],
        ) -> &'__slot mut [u8; 1024 * 1024] {
            slot
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
        _pin: PhantomPinned,
    }
    #[doc(hidden)]
    impl<'__pin> ::core::marker::Unpin for Foo
    where
        __Unpin<'__pin>: ::core::marker::Unpin,
    {}
    trait MustNotImplDrop {}
    #[expect(drop_bounds)]
    impl<T: ::core::ops::Drop> MustNotImplDrop for T {}
    impl MustNotImplDrop for Foo {}
    #[expect(non_camel_case_types)]
    trait UselessPinnedDropImpl_you_need_to_specify_PinnedDrop {}
    impl<T: ::pin_init::PinnedDrop> UselessPinnedDropImpl_you_need_to_specify_PinnedDrop
    for T {}
    impl UselessPinnedDropImpl_you_need_to_specify_PinnedDrop for Foo {}
};
