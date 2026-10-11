// SPDX-License-Identifier: Apache-2.0 OR MIT

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Data, DeriveInput, Field, Fields};

use crate::{diagnostics::ErrorGuaranteed, DiagCtxt};

pub(crate) fn derive(
    input: DeriveInput,
    dcx: &mut DiagCtxt,
) -> Result<TokenStream, ErrorGuaranteed> {
    let (fields, is_union) = match input.data {
        Data::Struct(data_struct) => (data_struct.fields, false),
        Data::Union(data_union) => (Fields::Named(data_union.fields), true),
        Data::Enum(data_enum) => {
            return Err(dcx.error(data_enum.enum_token, "cannot derive `Zeroable` for an enum"));
        }
    };
    let name = input.ident;
    let mut generics = input.generics;
    for param in generics.type_params_mut() {
        param.bounds.insert(0, parse_quote!(::pin_init::Zeroable));
    }
    let (impl_gen, ty_gen, whr) = generics.split_for_impl();
    let field_types: Vec<_> = fields.iter().map(|field| &field.ty).collect();
    let (safety, assertion) = if is_union {
        // A union is zeroable when at least one field is: the all-zero bit pattern is valid
        // if it is valid for any field.
        let mut checks = field_types
            .iter()
            .map(|ty| quote!(::pin_init::__internal::ZeroableCheck::<#ty>::new().check()));
        let Some(first) = checks.next() else {
            return Err(dcx.error(&name, "cannot derive `Zeroable` for a union without fields"));
        };
        let checks = checks.fold(first, |checks, check| quote!(#checks | #check));
        (
            quote! {
                // SAFETY: At least one field type is `Zeroable`, so the all-zero bit pattern is
                // a valid value of the union.
            },
            quote! {
                const _: () = {
                    use ::pin_init::__internal::ZeroableCheckFallback as _;
                    fn assert_zeroable_any<T: ::pin_init::__internal::IsTrue>(_: T) {}
                    fn ensure_zeroable #impl_gen ()
                        #whr
                    {
                        assert_zeroable_any(#checks);
                    }
                };
            },
        )
    } else {
        (
            quote! {
                // SAFETY: Every field type implements `Zeroable` and padding bytes may be zero.
            },
            quote! {
                const _: () = {
                    fn assert_zeroable<T: ?::core::marker::Sized + ::pin_init::Zeroable>() {}
                    fn ensure_zeroable #impl_gen ()
                        #whr
                    {
                        #(
                            assert_zeroable::<#field_types>();
                        )*
                    }
                };
            },
        )
    };
    Ok(quote! {
        #safety
        #[automatically_derived]
        unsafe impl #impl_gen ::pin_init::Zeroable for #name #ty_gen
            #whr
        {}
        #assertion
    })
}

pub(crate) fn maybe_derive(
    input: DeriveInput,
    dcx: &mut DiagCtxt,
) -> Result<TokenStream, ErrorGuaranteed> {
    let fields = match input.data {
        Data::Struct(data_struct) => data_struct.fields,
        Data::Union(data_union) => Fields::Named(data_union.fields),
        Data::Enum(data_enum) => {
            return Err(dcx.error(data_enum.enum_token, "cannot derive `Zeroable` for an enum"));
        }
    };
    let name = input.ident;
    let mut generics = input.generics;
    for param in generics.type_params_mut() {
        param.bounds.insert(0, parse_quote!(::pin_init::Zeroable));
    }
    for Field { ty, .. } in fields {
        generics
            .make_where_clause()
            .predicates
            // the `for<'__dummy>` HRTB makes this not error without the `trivial_bounds`
            // feature <https://github.com/rust-lang/rust/issues/48214#issuecomment-2557829956>.
            .push(parse_quote!(#ty: for<'__dummy> ::pin_init::Zeroable));
    }
    let (impl_gen, ty_gen, whr) = generics.split_for_impl();
    Ok(quote! {
        // SAFETY: Every field type implements `Zeroable` and padding bytes may be zero.
        #[automatically_derived]
        unsafe impl #impl_gen ::pin_init::Zeroable for #name #ty_gen
            #whr
        {}
    })
}
