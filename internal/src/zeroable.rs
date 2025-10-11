// SPDX-License-Identifier: GPL-2.0

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_quote, Data, DeriveInput, Field, Fields};

pub(crate) fn derive(input: DeriveInput) -> TokenStream {
    let fields = match input.data {
        Data::Struct(data_struct) => data_struct.fields,
        Data::Union(data_union) => Fields::Named(data_union.fields),
        Data::Enum(data_enum) => {
            return quote_spanned! {data_enum.enum_token.span=>
                ::core::compile_error!("cannot derive `Zeroable` for an enum");
            };
        }
    };
    let name = input.ident;
    let mut generics = input.generics;
    for param in generics.type_params_mut() {
        param.bounds.push(parse_quote!(::pin_init::Zeroable));
    }
    let (impl_gen, ty_gen, whr) = generics.split_for_impl();
    let field_type = fields.iter().map(|field| &field.ty);
    quote! {
        // SAFETY: Every field type implements `Zeroable` and padding bytes may be zero.
        #[automatically_derived]
        unsafe impl #impl_gen ::pin_init::Zeroable for #name #ty_gen
            #whr
        {}
        const _: () = {
            fn assert_zeroable<T: ?::core::marker::Sized + ::pin_init::Zeroable>() {}
            fn ensure_zeroable #impl_gen ()
                #whr
            {
                #(
                    assert_zeroable::<#field_type>();
                )*
            }
        };
    }
}

pub(crate) fn maybe_derive(input: DeriveInput) -> TokenStream {
    let fields = match input.data {
        Data::Struct(data_struct) => data_struct.fields,
        Data::Union(data_union) => Fields::Named(data_union.fields),
        Data::Enum(data_enum) => {
            return quote_spanned! {data_enum.enum_token.span=>
                compile_error!("cannot derive `Zeroable` for an enum");
            };
        }
    };
    let name = input.ident;
    let mut generics = input.generics;
    for Field { ty, .. } in fields {
        generics
            .make_where_clause()
            .predicates
            // the `for<'__dummy>` HRTB makes this not error without the `trivial_bounds`
            // feature <https://github.com/rust-lang/rust/issues/48214#issuecomment-2557829956>.
            .push(parse_quote!(#ty: for<'__dummy> ::pin_init::Zeroable));
    }
    let (impl_gen, ty_gen, whr) = generics.split_for_impl();
    quote! {
        // SAFETY: Every field type implements `Zeroable` and padding bytes may be zero.
        #[automatically_derived]
        unsafe impl #impl_gen ::pin_init::Zeroable for #name #ty_gen
            #whr
        {}
    }
}
