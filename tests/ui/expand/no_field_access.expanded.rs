use pin_init::*;
#[repr(C, packed)]
struct Foo {
    a: i8,
    b: i32,
    c: i32,
}
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
                        let c = -42;
                        unsafe { ::core::ptr::write(&raw mut (*slot).c, c) };
                    }
                    let __c_guard = unsafe {
                        ::pin_init::__internal::DropGuard::new(&raw mut (*slot).c)
                    };
                    {
                        let b = *c;
                        unsafe { ::core::ptr::write(&raw mut (*slot).b, b) };
                    }
                    let __b_guard = unsafe {
                        ::pin_init::__internal::DropGuard::new(&raw mut (*slot).b)
                    };
                    {
                        let a = 0;
                        unsafe { ::core::ptr::write(&raw mut (*slot).a, a) };
                    }
                    let __a_guard = unsafe {
                        ::pin_init::__internal::DropGuard::new(&raw mut (*slot).a)
                    };
                    ::core::mem::forget(__c_guard);
                    ::core::mem::forget(__b_guard);
                    ::core::mem::forget(__a_guard);
                    #[allow(unreachable_code, clippy::diverging_sub_expression)]
                    let _ = || unsafe {
                        ::core::ptr::write(
                            slot,
                            Foo {
                                c: ::core::panicking::panic("explicit panic"),
                                b: ::core::panicking::panic("explicit panic"),
                                a: ::core::panicking::panic("explicit panic"),
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
