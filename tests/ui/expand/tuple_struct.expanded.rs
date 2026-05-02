use core::marker::PhantomPinned;
use pin_init::*;
struct Foo<'a, T: Copy, const N: usize>(&'a mut [T; N], PhantomPinned, usize);
const _: () = {
    /// Pin-projections of [`Foo`]
    #[allow(dead_code, non_snake_case)]
    #[doc(hidden)]
    struct __Projection<'__this, 'a, T: Copy, const N: usize>(
        &'__this mut &'a mut [T; N],
        ::core::pin::Pin<&'__this mut PhantomPinned>,
        &'__this mut usize,
        ::core::marker::PhantomData<&'__this mut ()>,
    );
    /// Pin-projections of [`Foo`]
    #[allow(dead_code, non_snake_case)]
    #[doc(hidden)]
    struct __ProjectionLt<'__this, 'a, T: Copy, const N: usize>(
        &'__this mut &'a mut [T; N],
        ::core::pin::Pin<&'__this mut PhantomPinned>,
        &'__this mut usize,
        ::core::marker::PhantomData<&'__this mut ()>,
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
        fn project<'__this>(
            self: ::core::pin::Pin<&'__this mut Self>,
        ) -> __Projection<'__this, 'a, T, N> {
            let this = unsafe { ::core::pin::Pin::get_unchecked_mut(self) };
            __Projection(
                &mut this.0,
                unsafe { ::core::pin::Pin::new_unchecked(&mut this.1) },
                &mut this.2,
                ::core::marker::PhantomData,
            )
        }
        /// Pin-projects all fields of `Self` with proper lifetime.
        ///
        /// These fields are structurally pinned:
        /// - index `1`
        ///
        /// These fields are **not** structurally pinned:
        /// - index `0`
        /// - index `2`
        #[inline]
        fn with_project<'__this, R>(
            self: ::core::pin::Pin<&'__this mut Self>,
            f: impl ::core::ops::FnOnce(__ProjectionLt<'__this, 'a, T, N>) -> R,
        ) -> R {
            let this = unsafe { ::core::pin::Pin::get_unchecked_mut(self) };
            f(
                __ProjectionLt(
                    &mut this.0,
                    unsafe { ::core::pin::Pin::new_unchecked(&mut this.1) },
                    &mut this.2,
                    ::core::marker::PhantomData,
                ),
            )
        }
    }
    #[doc(hidden)]
    #[allow(non_snake_case)]
    struct __PinDataLt<'a, T: Copy, const N: usize> {
        _0: ::pin_init::__internal::PhantomInvariant<&'a mut [T; N]>,
        _1: ::pin_init::__internal::PhantomInvariant<PhantomPinned>,
        _2: ::pin_init::__internal::PhantomInvariant<usize>,
    }
    impl<'a, T: Copy, const N: usize> ::core::clone::Clone for __PinDataLt<'a, T, N> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<'a, T: Copy, const N: usize> ::core::marker::Copy for __PinDataLt<'a, T, N> {}
    #[allow(dead_code)]
    #[expect(clippy::missing_safety_doc)]
    impl<'a, T: Copy, const N: usize> __PinDataLt<'a, T, N> {
        /// # Safety
        ///
        /// - `slot` is valid and properly aligned.
        /// - `(*slot).#field_name` is properly aligned.
        /// - `(*slot).#field_name` points to uninitialized and exclusively accessed
        ///   memory.
        #[allow(non_snake_case)]
        #[inline(always)]
        unsafe fn _0(
            self,
            slot: *mut Foo<'a, T, N>,
        ) -> ::pin_init::__internal::Slot<
            ::pin_init::__internal::Unpinned,
            &'a mut [T; N],
        > {
            unsafe { ::pin_init::__internal::Slot::new(&raw mut (*slot).0 as _) }
        }
        /// # Safety
        ///
        /// - `slot` is valid and properly aligned.
        /// - `(*slot).#field_name` is properly aligned.
        /// - `(*slot).#field_name` points to uninitialized and exclusively accessed
        ///   memory.
        #[allow(non_snake_case)]
        #[inline(always)]
        unsafe fn _1(
            self,
            slot: *mut Foo<'a, T, N>,
        ) -> ::pin_init::__internal::Slot<
            ::pin_init::__internal::Pinned,
            PhantomPinned,
        > {
            unsafe { ::pin_init::__internal::Slot::new(&raw mut (*slot).1 as _) }
        }
        /// # Safety
        ///
        /// - `slot` is valid and properly aligned.
        /// - `(*slot).#field_name` is properly aligned.
        /// - `(*slot).#field_name` points to uninitialized and exclusively accessed
        ///   memory.
        #[allow(non_snake_case)]
        #[inline(always)]
        unsafe fn _2(
            self,
            slot: *mut Foo<'a, T, N>,
        ) -> ::pin_init::__internal::Slot<::pin_init::__internal::Unpinned, usize> {
            unsafe { ::pin_init::__internal::Slot::new(&raw mut (*slot).2 as _) }
        }
    }
    #[doc(hidden)]
    struct __ThePinData<'a, T: Copy, const N: usize> {
        __phantom: ::pin_init::__internal::PhantomInvariant<Foo<'a, T, N>>,
    }
    impl<'a, T: Copy, const N: usize> ::core::clone::Clone for __ThePinData<'a, T, N> {
        #[inline]
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<'a, T: Copy, const N: usize> ::core::marker::Copy for __ThePinData<'a, T, N> {}
    impl<'a, T: Copy, const N: usize> __ThePinData<'a, T, N> {
        /// Type inference helper function.
        #[inline(always)]
        fn __make_closure<__F, __E>(self, f: __F) -> __F
        where
            __F: ::core::ops::FnOnce(
                *mut Foo<'a, T, N>,
                __PinDataLt<'a, T, N>,
            ) -> ::core::result::Result<::pin_init::__internal::InitOk, __E>,
        {
            f
        }
        #[inline(always)]
        fn __with_lt(self) -> __PinDataLt<'a, T, N> {
            unsafe { ::core::mem::zeroed() }
        }
    }
    unsafe impl<'a, T: Copy, const N: usize> ::pin_init::__internal::HasPinData
    for Foo<'a, T, N> {
        type PinData = __ThePinData<'a, T, N>;
        #[inline]
        fn __pin_data(_: ::pin_init::__internal::InitData<Self>) -> Self::PinData {
            __ThePinData {
                __phantom: ::pin_init::__internal::PhantomInvariant::new(),
            }
        }
    }
    #[allow(dead_code, non_snake_case)]
    struct __Unpin<'a, T: Copy, const N: usize> {
        __phantom: ::pin_init::__internal::PhantomInvariant<Foo<'a, T, N>>,
        _1: PhantomPinned,
    }
    #[doc(hidden)]
    impl<'a, T: Copy, const N: usize> ::core::marker::Unpin for Foo<'a, T, N>
    where
        for<'__dummy> __Unpin<'a, T, N>: ::core::marker::Unpin,
    {}
    trait MustNotImplDrop {}
    impl<T: ::core::ops::Drop + ?::core::marker::Sized> MustNotImplDrop for T {}
    impl<'a, T: Copy, const N: usize> MustNotImplDrop for Foo<'a, T, N> {}
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
        let data = {
            use ::pin_init::__internal::HasInitData;
            Foo::__init_data()
        };
        let init = data
            .__make_closure::<
                _,
                ::core::convert::Infallible,
            >(move |slot, data_lt| {
                if true {
                    let mut ___0_guard = (unsafe {
                        ::pin_init::__internal::Slot::<
                            ::pin_init::__internal::Unpinned,
                            _,
                        >::new(&raw mut (*slot).0)
                    })
                        .write(&mut first);
                    let mut ___1_guard = (unsafe {
                        ::pin_init::__internal::Slot::<
                            ::pin_init::__internal::Unpinned,
                            _,
                        >::new(&raw mut (*slot).1)
                    })
                        .write(PhantomPinned);
                    let mut ___2_guard = (unsafe {
                        ::pin_init::__internal::Slot::<
                            ::pin_init::__internal::Unpinned,
                            _,
                        >::new(&raw mut (*slot).2)
                    })
                        .init(10)?;
                    ::core::mem::forget(___2_guard);
                    ::core::mem::forget(___1_guard);
                    ::core::mem::forget(___0_guard);
                } else {
                    let mut ___0_guard = (unsafe {
                        ::pin_init::__internal::Slot::<
                            ::pin_init::__internal::Unpinned,
                            _,
                        >::new(&raw mut (*slot).0)
                    })
                        .write(&mut first);
                    let mut ___1_guard = (unsafe {
                        ::pin_init::__internal::Slot::<
                            ::pin_init::__internal::Unpinned,
                            _,
                        >::new(&raw mut (*slot).1)
                    })
                        .write(PhantomPinned);
                    let mut ___2_guard = (unsafe {
                        ::pin_init::__internal::Slot::<
                            ::pin_init::__internal::Unpinned,
                            _,
                        >::new(&raw mut (*slot).2)
                    })
                        .init(10)?;
                    ::core::mem::forget(___2_guard);
                    ::core::mem::forget(___1_guard);
                    ::core::mem::forget(___0_guard);
                }
                #[allow(unreachable_code)]
                let _ = || unsafe {
                    let _ = &(*slot).0;
                    let _ = &(*slot).1;
                    let _ = &(*slot).2;
                    ::core::ptr::write(
                        slot,
                        Foo {
                            0: loop {},
                            1: loop {},
                            2: loop {},
                        },
                    )
                };
                Ok(unsafe { ::pin_init::__internal::InitOk::new() })
            });
        let init = move |
            slot,
        | -> ::core::result::Result<(), ::core::convert::Infallible> {
            init(slot, data.__with_lt()).map(|__InitOk| ())
        };
        unsafe { ::pin_init::init_from_closure::<_, ::core::convert::Infallible>(init) }
    };
    let mut second = [4u8, 5, 6];
    let _ = {
        let data = {
            use ::pin_init::__internal::HasInitData;
            Foo::__init_data()
        };
        let init = data
            .__make_closure::<
                _,
                ::core::convert::Infallible,
            >(move |slot, data_lt| {
                if true {
                    let mut ___0_guard = (unsafe {
                        ::pin_init::__internal::Slot::<
                            ::pin_init::__internal::Unpinned,
                            _,
                        >::new(&raw mut (*slot).0)
                    })
                        .write(&mut second);
                    let mut ___1_guard = (unsafe {
                        ::pin_init::__internal::Slot::<
                            ::pin_init::__internal::Unpinned,
                            _,
                        >::new(&raw mut (*slot).1)
                    })
                        .write(PhantomPinned);
                    let mut ___2_guard = (unsafe {
                        ::pin_init::__internal::Slot::<
                            ::pin_init::__internal::Unpinned,
                            _,
                        >::new(&raw mut (*slot).2)
                    })
                        .write(20);
                    ::core::mem::forget(___2_guard);
                    ::core::mem::forget(___1_guard);
                    ::core::mem::forget(___0_guard);
                } else {
                    let mut ___0_guard = (unsafe {
                        ::pin_init::__internal::Slot::<
                            ::pin_init::__internal::Unpinned,
                            _,
                        >::new(&raw mut (*slot).0)
                    })
                        .write(&mut second);
                    let mut ___1_guard = (unsafe {
                        ::pin_init::__internal::Slot::<
                            ::pin_init::__internal::Unpinned,
                            _,
                        >::new(&raw mut (*slot).1)
                    })
                        .write(PhantomPinned);
                    let mut ___2_guard = (unsafe {
                        ::pin_init::__internal::Slot::<
                            ::pin_init::__internal::Unpinned,
                            _,
                        >::new(&raw mut (*slot).2)
                    })
                        .write(20);
                    ::core::mem::forget(___2_guard);
                    ::core::mem::forget(___1_guard);
                    ::core::mem::forget(___0_guard);
                }
                #[allow(unreachable_code)]
                let _ = || unsafe {
                    let _ = &(*slot).0;
                    let _ = &(*slot).1;
                    let _ = &(*slot).2;
                    ::core::ptr::write(
                        slot,
                        Foo {
                            0: loop {},
                            1: loop {},
                            2: loop {},
                        },
                    )
                };
                Ok(unsafe { ::pin_init::__internal::InitOk::new() })
            });
        let init = move |
            slot,
        | -> ::core::result::Result<(), ::core::convert::Infallible> {
            init(slot, data.__with_lt()).map(|__InitOk| ())
        };
        unsafe { ::pin_init::init_from_closure::<_, ::core::convert::Infallible>(init) }
    };
}
