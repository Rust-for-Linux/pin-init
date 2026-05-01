use pin_init::*;
struct Foo {}
fn main() {
    let _ = {
        let __data = unsafe {
            use ::pin_init::__internal::HasInitData;
            Foo::__init_data()
        };
        let init = __data
            .__make_closure::<
                _,
                ::core::convert::Infallible,
            >(move |slot, __data| {
                #[allow(unreachable_code)]
                let _ = || unsafe { ::core::ptr::write(slot, Foo {}) };
                Ok(unsafe { ::pin_init::__internal::InitOk::new() })
            });
        let init = move |
            slot,
        | -> ::core::result::Result<(), ::core::convert::Infallible> {
            init(slot, __data.__with_lt()).map(|__InitOk| ())
        };
        unsafe { ::pin_init::init_from_closure::<_, ::core::convert::Infallible>(init) }
    };
}
