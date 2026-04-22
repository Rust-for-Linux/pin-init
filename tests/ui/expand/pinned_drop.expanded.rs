use core::{marker::PhantomPinned, pin::Pin};
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
        /// - `slot` points to a `#ident` field of a pinned struct that this `__ThePinData` describes.
        /// - `slot` is a valid, properly aligned and points to uninitialized and exclusively memory.
        unsafe fn array(
            self,
            slot: *mut [u8; 1024 * 1024],
        ) -> ::pin_init::__internal::Slot<
            ::pin_init::__internal::Unpinned,
            [u8; 1024 * 1024],
        > {
            unsafe { ::pin_init::__internal::Slot::new(slot) }
        }
        /// # Safety
        ///
        /// - `slot` points to a `#ident` field of a pinned struct that this `__ThePinData` describes.
        /// - `slot` is a valid, properly aligned and points to uninitialized and exclusively memory.
        unsafe fn _pin(
            self,
            slot: *mut PhantomPinned,
        ) -> ::pin_init::__internal::Slot<
            ::pin_init::__internal::Pinned,
            PhantomPinned,
        > {
            unsafe { ::pin_init::__internal::Slot::new(slot) }
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
