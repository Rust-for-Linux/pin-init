use pin_init::*;
struct Foo {}
fn main() {
    let _ = {
        let __data = unsafe {
            use ::pin_init::__internal::HasInitData;
            Foo::__init_data()
        };
        __data
            .__make_init::<
                _,
                ::core::convert::Infallible,
            >(move |slot| {
                #[allow(unreachable_code, clippy::diverging_sub_expression)]
                let _ = || unsafe { ::core::ptr::write(slot, Foo {}) };
                Ok(unsafe { ::pin_init::__internal::InitOk::new() })
            })
    };
}
