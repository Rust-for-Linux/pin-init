use pin_init::*;
struct Foo([u8; 3], i32);
/// Pin-projections of [`Foo`]
#[allow(dead_code)]
#[doc(hidden)]
struct FooProjection<'__pin> {
    _0: &'__pin mut [u8; 3],
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
            slot: *mut [u8; 3],
            init: impl ::pin_init::Init<[u8; 3], E>,
        ) -> ::core::result::Result<(), E> {
            unsafe { ::pin_init::Init::__init(init, slot) }
        }
        /// # Safety
        ///
        /// `slot` points at the field index `0` inside of `Foo`, which is pinned.
        unsafe fn __project_0<'__slot>(
            self,
            slot: &'__slot mut [u8; 3],
        ) -> &'__slot mut [u8; 3] {
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
fn main() {
    let _ = {
        struct __InitOk;
        let __data = unsafe {
            use ::pin_init::__internal::HasInitData;
            Foo::__init_data()
        };
        let init = ::pin_init::__internal::InitData::make_closure::<
            _,
            __InitOk,
            ::core::convert::Infallible,
        >(
            __data,
            move |slot| {
                {
                    struct __InitOk;
                    {
                        let _0 = [1, 2, 3];
                        unsafe { ::core::ptr::write(&raw mut (*slot).0, _0) };
                    }
                    #[allow(unused_variables)]
                    let _0 = unsafe { &mut (*slot).0 };
                    let ___0_guard = unsafe {
                        ::pin_init::__internal::DropGuard::new(&raw mut (*slot).0)
                    };
                    {
                        let init = 42;
                        unsafe { ::pin_init::Init::__init(init, &raw mut (*slot).1)? };
                    }
                    #[allow(unused_variables)]
                    let _1 = unsafe { &mut (*slot).1 };
                    let ___1_guard = unsafe {
                        ::pin_init::__internal::DropGuard::new(&raw mut (*slot).1)
                    };
                    ::core::mem::forget(___0_guard);
                    ::core::mem::forget(___1_guard);
                    #[allow(unreachable_code, clippy::diverging_sub_expression)]
                    let _ = || unsafe {
                        ::core::ptr::write(
                            slot,
                            Foo {
                                0: ::core::panicking::panic("explicit panic"),
                                1: ::core::panicking::panic("explicit panic"),
                            },
                        )
                    };
                }
                Ok(__InitOk)
            },
        );
        let init = move |
            slot,
        | -> ::core::result::Result<(), ::core::convert::Infallible> {
            init(slot).map(|__InitOk| ())
        };
        let init = unsafe {
            ::pin_init::init_from_closure::<_, ::core::convert::Infallible>(init)
        };
        #[allow(
            clippy::let_and_return,
            reason = "some clippy versions warn about the let binding"
        )] init
    };
}
