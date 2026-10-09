use pin_init::*;
struct Foo {}
fn main() {
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
                if true {} else {}
                #[allow(unreachable_code)]
                let _ = || unsafe { ::core::ptr::write(slot, Foo {}) };
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
