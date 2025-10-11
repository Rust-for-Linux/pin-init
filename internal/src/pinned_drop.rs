// SPDX-License-Identifier: Apache-2.0 OR MIT

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse::Nothing, parse_quote, spanned::Spanned, ImplItem, ItemImpl, Token};

pub(crate) fn pinned_drop(_args: Nothing, mut input: ItemImpl) -> TokenStream {
    let mut errors = vec![];
    if let Some(unsafety) = input.unsafety {
        errors.push(quote_spanned! {unsafety.span=>
            ::core::compile_error!("implementing `PinnedDrop` is safe");
        });
    }
    input.unsafety = Some(Token![unsafe](input.impl_token.span));
    match &mut input.trait_ {
        Some((not, path, _for)) => {
            if let Some(not) = not {
                errors.push(quote_spanned! {not.span=>
                    ::core::compile_error!("cannot implement `!PinnedDrop`");
                });
            }
            for (seg, expected) in path
                .segments
                .iter()
                .rev()
                .zip(["PinnedDrop", "pin_init", ""])
            {
                if expected.is_empty() || seg.ident != expected {
                    errors.push(quote_spanned! {seg.span()=>
                        ::core::compile_error!("bad import path for `PinnedDrop`");
                    });
                }
                if !seg.arguments.is_none() {
                    errors.push(quote_spanned! {seg.arguments.span()=>
                        ::core::compile_error!("unexpected arguments for `PinnedDrop` path");
                    });
                }
            }
            *path = parse_quote!(::pin_init::PinnedDrop);
        }
        None => errors.push(quote_spanned! {input.impl_token.span=>
            ::core::compile_error!("expected `impl ... PinnedDrop for ...`, got inherent impl");
        }),
    }
    for item in &mut input.items {
        if let ImplItem::Fn(fn_item) = item {
            if fn_item.sig.ident == "drop" {
                fn_item
                    .sig
                    .inputs
                    .push(parse_quote!(_: ::pin_init::__internal::OnlyCallFromDrop));
            }
        }
    }
    quote!(#(#errors)* #input)
}
