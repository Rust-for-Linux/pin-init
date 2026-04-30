use core::marker::PhantomPinned;
use pin_init::*;
struct Foo<'a, T: Copy, const N: usize>(&'a mut [T; N], PhantomPinned, usize);
/// Pin-projections of [`Foo`]
#[allow(dead_code)]
#[doc(hidden)]
struct FooProjection<'__pin, 'a, T: Copy, const N: usize> {
    _0: &'__pin mut &'a mut [T; N],
    _1: ::core::pin::Pin<&'__pin mut PhantomPinned>,
    _2: &'__pin mut usize,
    ___pin_phantom_data: ::core::marker::PhantomData<&'__pin mut ()>,
}
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
        FooProjection {
            _0: &mut this.0,
            _1: unsafe { ::core::pin::Pin::new_unchecked(&mut this.1) },
            _2: &mut this.2,
            ___pin_phantom_data: ::core::marker::PhantomData,
        }
    }
}
const _: () = {
    #[doc(hidden)]
    struct __ThePinData<'a, T: Copy, const N: usize> {
        __phantom: ::core::marker::PhantomData<fn(Foo<'a, T, N>) -> Foo<'a, T, N>>,
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
        /// # Safety
        ///
        /// - `slot` is a valid pointer to uninitialized memory.
        /// - the caller does not touch `slot` when `Err` is returned, they are only permitted
        ///   to deallocate.
        unsafe fn _0<E>(
            self,
            slot: *mut &'a mut [T; N],
            init: impl ::pin_init::Init<&'a mut [T; N], E>,
        ) -> ::core::result::Result<(), E> {
            unsafe { ::pin_init::Init::__init(init, slot) }
        }
        /// # Safety
        ///
        /// `slot` points at the field index `0` inside of `Foo`, which is pinned.
        unsafe fn __project_0<'__slot>(
            self,
            slot: &'__slot mut &'a mut [T; N],
        ) -> &'__slot mut &'a mut [T; N] {
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
            slot: *mut PhantomPinned,
            init: impl ::pin_init::PinInit<PhantomPinned, E>,
        ) -> ::core::result::Result<(), E> {
            unsafe { ::pin_init::PinInit::__pinned_init(init, slot) }
        }
        /// # Safety
        ///
        /// `slot` points at the field index `1` inside of `Foo`, which is pinned.
        unsafe fn __project_1<'__slot>(
            self,
            slot: &'__slot mut PhantomPinned,
        ) -> ::core::pin::Pin<&'__slot mut PhantomPinned> {
            unsafe { ::core::pin::Pin::new_unchecked(slot) }
        }
        /// # Safety
        ///
        /// - `slot` is a valid pointer to uninitialized memory.
        /// - the caller does not touch `slot` when `Err` is returned, they are only permitted
        ///   to deallocate.
        unsafe fn _2<E>(
            self,
            slot: *mut usize,
            init: impl ::pin_init::Init<usize, E>,
        ) -> ::core::result::Result<(), E> {
            unsafe { ::pin_init::Init::__init(init, slot) }
        }
        /// # Safety
        ///
        /// `slot` points at the field index `2` inside of `Foo`, which is pinned.
        unsafe fn __project_2<'__slot>(
            self,
            slot: &'__slot mut usize,
        ) -> &'__slot mut usize {
            slot
        }
    }
    unsafe impl<'a, T: Copy, const N: usize> ::pin_init::__internal::HasPinData
    for Foo<'a, T, N> {
        type PinData = __ThePinData<'a, T, N>;
        unsafe fn __pin_data() -> Self::PinData {
            __ThePinData {
                __phantom: ::core::marker::PhantomData,
            }
        }
    }
    unsafe impl<'a, T: Copy, const N: usize> ::pin_init::__internal::PinData
    for __ThePinData<'a, T, N> {
        type Datee = Foo<'a, T, N>;
    }
    #[allow(dead_code)]
    struct __Unpin<'__pin, 'a, T: Copy, const N: usize> {
        __phantom_pin: ::core::marker::PhantomData<fn(&'__pin ()) -> &'__pin ()>,
        __phantom: ::core::marker::PhantomData<fn(Foo<'a, T, N>) -> Foo<'a, T, N>>,
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
fn main() {
    let mut first = [1u8, 2, 3];
    let _ = {
        let __data = unsafe {
            use ::pin_init::__internal::HasInitData;
            Foo::__init_data()
        };
        let init = ::pin_init::__internal::InitData::make_closure::<
            _,
            ::core::convert::Infallible,
        >(
            __data,
            move |slot| {
                {
                    #[allow(clippy::just_underscores_and_digits)]
                    let _0 = &mut first;
                    unsafe { ::core::ptr::write(&raw mut (*slot).0, _0) };
                }
                #[allow(unused_variables)]
                #[allow(clippy::just_underscores_and_digits)]
                let _0 = unsafe { &mut (*slot).0 };
                let ___0_guard = unsafe {
                    ::pin_init::__internal::DropGuard::new(&raw mut (*slot).0)
                };
                {
                    #[allow(clippy::just_underscores_and_digits)]
                    let _1 = PhantomPinned;
                    unsafe { ::core::ptr::write(&raw mut (*slot).1, _1) };
                }
                #[allow(unused_variables)]
                #[allow(clippy::just_underscores_and_digits)]
                let _1 = unsafe { &mut (*slot).1 };
                let ___1_guard = unsafe {
                    ::pin_init::__internal::DropGuard::new(&raw mut (*slot).1)
                };
                {
                    let init = 10;
                    unsafe { ::pin_init::Init::__init(init, &raw mut (*slot).2)? };
                }
                #[allow(unused_variables)]
                #[allow(clippy::just_underscores_and_digits)]
                let _2 = unsafe { &mut (*slot).2 };
                let ___2_guard = unsafe {
                    ::pin_init::__internal::DropGuard::new(&raw mut (*slot).2)
                };
                ::core::mem::forget(___0_guard);
                ::core::mem::forget(___1_guard);
                ::core::mem::forget(___2_guard);
                #[allow(unreachable_code, clippy::diverging_sub_expression)]
                let _ = || unsafe {
                    ::core::ptr::write(
                        slot,
                        Foo {
                            0: ::core::panicking::panic("explicit panic"),
                            1: ::core::panicking::panic("explicit panic"),
                            2: ::core::panicking::panic("explicit panic"),
                        },
                    )
                };
                Ok(unsafe { ::pin_init::__internal::InitOk::new() })
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
    let mut second = [4u8, 5, 6];
    let _ = {
        let __data = unsafe {
            use ::pin_init::__internal::HasInitData;
            Foo::__init_data()
        };
        let init = ::pin_init::__internal::InitData::make_closure::<
            _,
            ::core::convert::Infallible,
        >(
            __data,
            move |slot| {
                {
                    #[allow(clippy::just_underscores_and_digits)]
                    let _0 = &mut second;
                    unsafe { ::core::ptr::write(&raw mut (*slot).0, _0) };
                }
                #[allow(unused_variables)]
                #[allow(clippy::just_underscores_and_digits)]
                let _0 = unsafe { &mut (*slot).0 };
                let ___0_guard = unsafe {
                    ::pin_init::__internal::DropGuard::new(&raw mut (*slot).0)
                };
                {
                    #[allow(clippy::just_underscores_and_digits)]
                    let _1 = PhantomPinned;
                    unsafe { ::core::ptr::write(&raw mut (*slot).1, _1) };
                }
                #[allow(unused_variables)]
                #[allow(clippy::just_underscores_and_digits)]
                let _1 = unsafe { &mut (*slot).1 };
                let ___1_guard = unsafe {
                    ::pin_init::__internal::DropGuard::new(&raw mut (*slot).1)
                };
                {
                    #[allow(clippy::just_underscores_and_digits)]
                    let _2 = 20;
                    unsafe { ::core::ptr::write(&raw mut (*slot).2, _2) };
                }
                #[allow(unused_variables)]
                #[allow(clippy::just_underscores_and_digits)]
                let _2 = unsafe { &mut (*slot).2 };
                let ___2_guard = unsafe {
                    ::pin_init::__internal::DropGuard::new(&raw mut (*slot).2)
                };
                ::core::mem::forget(___0_guard);
                ::core::mem::forget(___1_guard);
                ::core::mem::forget(___2_guard);
                #[allow(unreachable_code, clippy::diverging_sub_expression)]
                let _ = || unsafe {
                    ::core::ptr::write(
                        slot,
                        Foo {
                            0: ::core::panicking::panic("explicit panic"),
                            1: ::core::panicking::panic("explicit panic"),
                            2: ::core::panicking::panic("explicit panic"),
                        },
                    )
                };
                Ok(unsafe { ::pin_init::__internal::InitOk::new() })
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
