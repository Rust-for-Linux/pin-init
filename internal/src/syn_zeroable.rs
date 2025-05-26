// SPDX-License-Identifier: Apache-2.0 OR MIT

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Data, DataStruct, DataUnion, DeriveInput, Error, GenericParam,
    Generics, Ident, Result, Type, TypeParam, WherePredicate,
};

pub(crate) fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let raw = input.clone().into();
    match do_derive(&mut parse_macro_input!(input as DeriveInput), raw) {
        Ok((generics, ident, field_ty)) => expand(generics, ident, field_ty),
        Err(e) => e.into_compile_error(),
    }
    .into()
}

pub(crate) fn maybe_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let raw = input.clone().into();
    match do_derive(&mut parse_macro_input!(input as DeriveInput), raw) {
        Ok((generics, ident, field_ty)) => expand_maybe(generics, ident, field_ty),
        Err(e) => e.into_compile_error(),
    }
    .into()
}

fn do_derive(
    DeriveInput {
        ident,
        ref mut generics,
        data,
        ..
    }: &mut DeriveInput,
    raw_input: TokenStream,
) -> Result<(&mut Generics, &Ident, impl Iterator<Item = Type>)> {
    let field_ty = match data {
        Data::Struct(DataStruct { fields, .. }) => {
            fields.iter().map(|f| f.ty.clone()).collect::<Vec<_>>()
        }
        Data::Union(DataUnion { fields, .. }) => fields
            .named
            .iter()
            .map(|f| f.ty.clone())
            .collect::<Vec<_>>(),
        _ => {
            return Err(Error::new_spanned(
                raw_input,
                "`Zeroable` can only be derived for structs and unions.",
            ))
        }
    };
    let zeroable_bounds = generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(TypeParam { ident, .. }) => {
                Some(parse_quote!(#ident: ::pin_init::Zeroable))
            }
            _ => None,
        })
        .collect::<Vec<WherePredicate>>();
    generics
        .make_where_clause()
        .predicates
        .extend(zeroable_bounds);
    Ok((generics, ident, field_ty.into_iter()))
}

fn expand(
    generics: &mut Generics,
    ident: &Ident,
    field_ty: impl Iterator<Item = Type>,
) -> TokenStream {
    let (impl_generics, ty_generics, whr) = generics.split_for_impl();
    quote! {
        // SAFETY: Every field type implements `Zeroable` and padding bytes may be zero.
        #[automatically_derived]
        unsafe impl #impl_generics ::pin_init::Zeroable for #ident #ty_generics
            #whr
        {}
        const _: () = {
            fn assert_zeroable<T: ?::core::marker::Sized + ::pin_init::Zeroable>() {}
            fn ensure_zeroable #impl_generics ()
                #whr
            {
                #(assert_zeroable::<#field_ty>();)*
            }
        };
    }
}

fn expand_maybe(
    generics: &mut Generics,
    ident: &Ident,
    field_ty: impl Iterator<Item = Type>,
) -> TokenStream {
    generics
        .make_where_clause()
        .predicates
        .extend(field_ty.map(|bounded_ty| -> WherePredicate {
            parse_quote!(
                // the `for<'__dummy>` HRTB makes this not error without the `trivial_bounds`
                // feature <https://github.com/rust-lang/rust/issues/48214#issuecomment-2557829956>.
                #bounded_ty: for<'__dummy> ::pin_init::Zeroable
            )
        }));
    let (impl_generics, ty_generics, whr) = generics.split_for_impl();
    quote! {
        // SAFETY: Every field type implements `Zeroable` and padding bytes may be zero.
        #[automatically_derived]
        unsafe impl #impl_generics ::pin_init::Zeroable for #ident #ty_generics
            #whr
        {}
    }
}
