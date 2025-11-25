#[proc_macro_derive(Dummy, attributes(dummy_attr))]
pub fn derive_device(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}
