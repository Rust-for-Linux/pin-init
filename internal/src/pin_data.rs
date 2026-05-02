// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse::{End, Nothing, Parse},
    parse_quote, parse_quote_spanned,
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::VisitMut,
    Attribute, Field, Fields, GenericParam, Generics, Ident, Item, ItemStruct, Lifetime,
    LifetimeParam, PathSegment, Token, Type, TypePath, Visibility, WhereClause,
};

use crate::{
    diagnostics::{DiagCtxt, ErrorGuaranteed},
    util::{Binder, LifetimeExt, TypeExt},
};

pub(crate) mod kw {
    syn::custom_keyword!(PinnedDrop);
}

pub(crate) enum Args {
    Nothing(Nothing),
    #[allow(dead_code)]
    PinnedDrop(kw::PinnedDrop),
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let lh = input.lookahead1();
        if lh.peek(End) {
            input.parse().map(Self::Nothing)
        } else if lh.peek(kw::PinnedDrop) {
            input.parse().map(Self::PinnedDrop)
        } else {
            Err(lh.error())
        }
    }
}

impl ToTokens for Args {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Nothing(_) => (),
            Self::PinnedDrop(kw) => kw.to_tokens(tokens),
        }
    }
}

/// Annotation that a field is referenced.
struct Borrow {
    mutable: Option<Token![mut]>,
    /// Name of the field that this lifetime can reference.
    field: Ident,
}

impl Parse for Borrow {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        Ok(Borrow {
            mutable: input.parse()?,
            field: input.parse()?,
        })
    }
}

#[expect(unused)]
enum Variance {
    Covariant(Span),
    NotCovariant(Span),
}

impl Variance {
    fn parse(dcx: &mut DiagCtxt, attrs: &mut Vec<Attribute>, span: Span) -> Self {
        let mut attrs = attrs.extract_if(.., |attr| {
            attr.path().is_ident("covariant") || attr.path().is_ident("not_covariant")
        });

        let result = match attrs.next() {
            None => {
                // By default, infer covariance.
                Variance::Covariant(span)
            }
            Some(attr) => {
                if attr.path().is_ident("covariant") {
                    Variance::Covariant(attr.path().span())
                } else {
                    Variance::NotCovariant(attr.path().span())
                }
            }
        };

        // Emit error on redundant specifications.
        for attr in attrs {
            dcx.error(attr.path(), "variance marker can only be specified once");
        }

        result
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Borrowed {
    Shared,
    Mutable,
}

struct FieldInfo<'a> {
    field: &'a Field,
    pinned: bool,
    borrowed: Option<Borrowed>,
    variance: Option<Variance>,
    ty: Binder<Type>,
}

pub(crate) fn pin_data(
    args: Args,
    input: Item,
    dcx: &mut DiagCtxt,
) -> Result<TokenStream, ErrorGuaranteed> {
    let mut struct_ = match input {
        Item::Struct(struct_) => struct_,
        Item::Enum(enum_) => {
            return Err(dcx.error(
                enum_.enum_token,
                "`#[pin_data]` only supports structs for now",
            ));
        }
        Item::Union(union) => {
            return Err(dcx.error(
                union.union_token,
                "`#[pin_data]` only supports structs for now",
            ));
        }
        rest => {
            return Err(dcx.error(
                rest,
                "`#[pin_data]` can only be applied to struct, enum and union definitions",
            ));
        }
    };

    // Handling cfg can gets very complicated, especially for tuple structs.
    // Therefore, resolve all field cfgs first before continuing.
    for (field_idx, field) in struct_.fields.iter_mut().enumerate() {
        let cfg: Vec<_> = field
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("cfg"))
            .map(|a| {
                a.parse_args::<TokenStream>()
                    .expect("parse as token stream cannot fail")
            })
            .collect();

        if cfg.is_empty() {
            continue;
        }

        field.attrs.retain(|a| !a.path().is_ident("cfg"));
        let cfg_true_struct = quote!(#struct_);

        let punctuated = match &mut struct_.fields {
            Fields::Named(fields) => &mut fields.named,
            Fields::Unnamed(fields) => &mut fields.unnamed,
            Fields::Unit => unreachable!(),
        };
        *punctuated = std::mem::take(punctuated)
            .into_pairs()
            .enumerate()
            .filter(|&(i, _)| i != field_idx)
            .map(|(_, p)| p)
            .collect();
        let cfg_false_struct = quote!(#struct_);

        // Resolve one field at a time until we've got no more field cfgs.
        return Ok(quote!(
            #[cfg(all(#(#cfg,)*))]
            #[::pin_init::pin_data(#args)]
            #cfg_true_struct

            #[cfg(not(all(#(#cfg,)*)))]
            #[::pin_init::pin_data(#args)]
            #cfg_false_struct
        ));
    }

    // The generics might contain the `Self` type. Since this macro will define a new type with the
    // same generics and bounds, this poses a problem: `Self` will refer to the new type as opposed
    // to this struct definition. Therefore we have to replace `Self` with the concrete name.
    let mut replacer = {
        let name = &struct_.ident;
        let (_, ty_generics, _) = struct_.generics.split_for_impl();
        SelfReplacer(parse_quote!(#name #ty_generics))
    };
    replacer.visit_generics_mut(&mut struct_.generics);
    replacer.visit_fields_mut(&mut struct_.fields);

    // Move `fields` out from the struct.
    let mut fields = struct_.fields;
    struct_.fields = Fields::Unit;

    // Collect all bound lifetimes from generics.
    let bound_lifetimes: BTreeSet<&Lifetime> =
        struct_.generics.lifetimes().map(|x| &x.lifetime).collect();

    // Keep track on how fields are borrowed and where.
    let mut borrow_info = BTreeMap::<_, (bool, Vec<Span>)>::new();

    let mut fields: Vec<FieldInfo<'_>> = fields
        .iter_mut()
        .map(|field| {
            let len = field.attrs.len();
            field.attrs.retain(|a| !a.path().is_ident("pin"));
            let pinned_count = len - field.attrs.len();
            if pinned_count > 1 {
                dcx.error(&field, "#[pin] attribute specified more than once");
            }

            assert!(
                !field.attrs.iter().any(|a| a.path().is_ident("cfg")),
                "cfgs should be all resolved at this point"
            );

            let ident = field.ident.as_ref().unwrap();

            // Parse `#[covariant]` and `#[not_covariant]` markers.
            let variance = Variance::parse(dcx, &mut field.attrs, ident.span());

            // Parse `#[borrows]` attribute or infer it from the type.
            let borrows: Punctuated<Borrow, Token![,]> = field
                .attrs
                .extract_if(.., |attr| attr.path().is_ident("borrows"))
                .filter_map(
                    |attr| match attr.parse_args_with(Punctuated::parse_terminated) {
                        Ok(v) => Some(v),
                        Err(err) => {
                            dcx.error(attr, err);
                            None
                        }
                    },
                )
                .next()
                .unwrap_or_else(|| {
                    // If no annotations are attached, infer lifetime based on the field referenced,
                    // although bound lifetimes from struct generics should take priority.
                    //
                    // For example,
                    // ```
                    // struct Foo<'a> {
                    //     bar: &'a (),
                    //     a: u32,
                    // }
                    // ```
                    // would not be inferred as self-referential because `'a` is already bound by the
                    // struct generics.
                    //
                    // Note that we cannot reliably determine what type of borrow is needed on fields.
                    // More commonly a shared borrow is required, so it is inferred as such. Mutable
                    // borrow would require explicit user annotation.
                    field
                        .ty
                        .unbound_lifetimes()
                        .difference(&bound_lifetimes)
                        .map(|&lt| Borrow {
                            mutable: None,
                            field: lt.ident.clone(),
                        })
                        .collect()
                });

            // Update borrow information.
            for borrow in borrows.iter() {
                let entry = borrow_info.entry(borrow.field.clone()).or_default();
                entry.0 |= borrow.mutable.is_some();
                entry.1.push(borrow.field.span());
            }

            let ty = Binder::new(
                borrows
                    .iter()
                    .map(|field_ref| Lifetime::from_ident(&field_ref.field))
                    .collect(),
                field.ty.clone(),
            );

            FieldInfo {
                field: &*field,
                pinned: pinned_count != 0,
                borrowed: None,
                variance: if borrows.is_empty() {
                    None
                } else {
                    Some(variance)
                },
                ty,
            }
        })
        .collect();

    for field in fields.iter_mut() {
        let ident = field.field.ident.as_ref().unwrap();

        if !field.pinned && is_phantom_pinned(&field.field.ty) {
            dcx.warn(
                field.field,
                format!(
                    "The field `{}` of type `PhantomPinned` only has an effect \
                    if it has the `#[pin]` attribute",
                    ident,
                ),
            );
        }

        // Update `borrowed` now we have all borrow information recorded.
        // This is not `remove()` because fields might be `#[cfg]` on so identifier names are not unique.
        if let Some((mutable, users)) = borrow_info.get_mut(ident) {
            field.borrowed = Some(if *mutable {
                Borrowed::Mutable
            } else {
                Borrowed::Shared
            });

            users.clear();
        }

        // We have a limitation of up to 4 referenced lifetime in a type.
        // See `ForLt4` and `SelfRef` in `__internal.rs` on why this limitation exists.
        if field.ty.bound.len() > 4 {
            dcx.error(
                &field.ty.bound[4],
                "at most 4 lifetimes can be referenced from a struct at a time",
            );
        }
    }

    // For any residual users in `borrow_info`, the lifetime mention is invalid and report as such.
    for (name, (_, users)) in borrow_info {
        for user in users {
            dcx.error(
                user,
                format!("`{name}` is neither a lifetime in generics nor a field name"),
            );
        }
    }

    let struct_def = generate_struct_def(&struct_, &fields);
    let unpin_impl = generate_unpin_impl(&struct_.ident, &struct_.generics, &fields);
    let drop_impl = generate_drop_impl(&struct_.ident, &struct_.generics, args);
    let drop_check = generate_drop_check(dcx, &struct_, &fields);
    let projections =
        generate_projections(&struct_.vis, &struct_.ident, &struct_.generics, &fields);
    let the_pin_data =
        generate_the_pin_data(&struct_.vis, &struct_.ident, &struct_.generics, &fields);

    Ok(quote! {
        #struct_def
        // We put the rest into this const item, because it then will not be accessible to anything
        // outside.
        const _: () = {
            #drop_check
            #projections
            #the_pin_data
            #unpin_impl
            #drop_impl
        };
    })
}

fn is_phantom_pinned(ty: &Type) -> bool {
    match ty {
        Type::Path(TypePath { qself: None, path }) => {
            // Cannot possibly refer to `PhantomPinned` (except alias, but that's on the user).
            if path.segments.len() > 3 {
                return false;
            }
            // If there is a `::`, then the path needs to be `::core::marker::PhantomPinned` or
            // `::std::marker::PhantomPinned`.
            if path.leading_colon.is_some() && path.segments.len() != 3 {
                return false;
            }
            let expected: Vec<&[&str]> = vec![&["PhantomPinned"], &["marker"], &["core", "std"]];
            for (actual, expected) in path.segments.iter().rev().zip(expected) {
                if !actual.arguments.is_empty() || expected.iter().all(|e| actual.ident != e) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

fn generate_struct_def(
    ItemStruct {
        attrs,
        vis,
        struct_token,
        ident,
        generics,
        fields: _,
        semi_token,
    }: &ItemStruct,
    fields: &[FieldInfo<'_>],
) -> TokenStream {
    let mut generated_fields = Vec::new();

    for field in fields {
        let Field {
            attrs,
            vis,
            mutability: _,
            ident,
            colon_token,
            ty: _,
        } = &field.field;

        let mut ty = field.ty.value.to_token_stream();

        if field.variance.is_some() {
            let bound = field.ty.for_bound_4();
            let bound_lt = bound.lifetimes.iter();
            ty = parse_quote!(::pin_init::__internal::SelfRef<
                ::pin_init::__internal::ForLtImpl<
                    dyn #bound ::pin_init::__internal::WithLt4<#(#bound_lt,)* Of = #ty>
                >
            >);
        };

        // For mutably referenced items, it is possible to access `&struct.field` through `Pin<&mut
        // Struct>`, which conflicts with the mutable access possible via the mutable borrow when
        // constructing. Wrap the type behind `UnsafePinned` so it's not UB to have both and it also
        // blocks user from doing anything wiht the value (safely).
        if field.borrowed == Some(Borrowed::Mutable) {
            ty = parse_quote!(::pin_init::__internal::UnsafePinned<#ty>);
        }

        generated_fields.push(quote! {
           #(#attrs)* #vis #ident #colon_token #ty
        });
    }

    let whr = &generics.where_clause;

    quote!(
        #(#attrs)*
        #vis
        #struct_token #ident #generics #whr {
            #(#generated_fields,)*
        }
        #semi_token
    )
}

fn generate_unpin_impl(
    ident: &Ident,
    generics: &Generics,
    fields: &[FieldInfo<'_>],
) -> TokenStream {
    let (_, ty_generics, _) = generics.split_for_impl();
    let mut generics_with_pin_lt = generics.clone();
    generics_with_pin_lt.params.insert(0, parse_quote!('__pin));
    generics_with_pin_lt.make_where_clause();
    let (
        impl_generics_with_pin_lt,
        ty_generics_with_pin_lt,
        Some(WhereClause {
            where_token,
            predicates,
        }),
    ) = generics_with_pin_lt.split_for_impl()
    else {
        unreachable!()
    };

    let pinned_fields = if fields.iter().any(|f| f.borrowed.is_some()) {
        // Self-referential structs must always be pinned.
        quote!(
            __phantom_pinned: ::core::marker::PhantomPinned,
        )
    } else {
        let pinned_fields = fields.iter().filter(|f| f.pinned).map(|f| {
            let ident = f.field.ident.as_ref().unwrap();
            let ty = &f.field.ty;
            quote!(
                #ident: #ty
            )
        });
        quote!(#(#pinned_fields,)*)
    };

    quote! {
        // This struct will be used for the unpin analysis. It is needed, because only structurally
        // pinned fields are relevant whether the struct should implement `Unpin`.
        #[allow(
            dead_code, // The fields below are never used.
            non_snake_case // The warning will be emitted on the struct definition.
        )]
        struct __Unpin #generics_with_pin_lt
        #where_token
            #predicates
        {
            __phantom_pin: ::pin_init::__internal::PhantomInvariantLifetime<'__pin>,
            __phantom: ::pin_init::__internal::PhantomInvariant<#ident #ty_generics>,
            #pinned_fields
        }

        #[doc(hidden)]
        impl #impl_generics_with_pin_lt ::core::marker::Unpin for #ident #ty_generics
        #where_token
            __Unpin #ty_generics_with_pin_lt: ::core::marker::Unpin,
            #predicates
        {}
    }
}

fn generate_drop_impl(ident: &Ident, generics: &Generics, args: Args) -> TokenStream {
    let (impl_generics, ty_generics, whr) = generics.split_for_impl();
    let has_pinned_drop = matches!(args, Args::PinnedDrop(_));
    // We need to disallow normal `Drop` implementation, the exact behavior depends on whether
    // `PinnedDrop` was specified in `args`.
    if has_pinned_drop {
        // When `PinnedDrop` was specified we just implement `Drop` and delegate.
        quote! {
            impl #impl_generics ::core::ops::Drop for #ident #ty_generics
                #whr
            {
                fn drop(&mut self) {
                    // SAFETY: Since this is a destructor, `self` will not move after this function
                    // terminates, since it is inaccessible.
                    let pinned = unsafe { ::core::pin::Pin::new_unchecked(self) };
                    // SAFETY: Since this is a drop function, we can create this token to call the
                    // pinned destructor of this type.
                    let token = unsafe { ::pin_init::__internal::OnlyCallFromDrop::new() };
                    ::pin_init::PinnedDrop::drop(pinned, token);
                }
            }
        }
    } else {
        // When no `PinnedDrop` was specified, then we have to prevent implementing drop.
        quote! {
            // We prevent this by creating a trait that will be implemented for all types implementing
            // `Drop`. Additionally we will implement this trait for the struct leading to a conflict,
            // if it also implements `Drop`
            trait MustNotImplDrop {}
            impl<T: ::core::ops::Drop + ?::core::marker::Sized> MustNotImplDrop for T {}
            impl #impl_generics MustNotImplDrop for #ident #ty_generics
                #whr
            {}
            // We also take care to prevent users from writing a useless `PinnedDrop` implementation.
            // They might implement `PinnedDrop` correctly for the struct, but forget to give
            // `PinnedDrop` as the parameter to `#[pin_data]`.
            trait UselessPinnedDropImpl_you_need_to_specify_PinnedDrop {}
            impl<T: ::pin_init::PinnedDrop + ?::core::marker::Sized>
                UselessPinnedDropImpl_you_need_to_specify_PinnedDrop for T {}
            impl #impl_generics
                UselessPinnedDropImpl_you_need_to_specify_PinnedDrop for #ident #ty_generics
                #whr
            {}
        }
    }
}

fn generate_drop_check(
    dcx: &mut DiagCtxt,
    struct_: &ItemStruct,
    fields: &[FieldInfo<'_>],
) -> TokenStream {
    let struct_name = &struct_.ident;
    // If the struct is not self-referential then we can just skip. However, still leave a
    // `__DropCheck` type around which can be used to capture all field lifetimes by other
    // generated code.
    if fields.iter().all(|f| f.borrowed.is_none()) {
        return quote!(
            use #struct_name as __DropCheck;
        );
    }

    // Make sure fields are dropped earlier than the fields that they borrow.
    //
    // Note that this only order fields and their borrow, and not establish the order between two
    // borrows. The latter is checked below with `__drop_check`. We could also use `__drop_check`
    // to perform what we check here, but it'll require synthesize lifetimes for more fields and
    // will emit a less clear error message.
    for (i, field) in fields.iter().enumerate() {
        let ident = field.field.ident.as_ref().unwrap();
        for lt in &field.ty.bound {
            let borrowed_field = &lt.ident;

            fields
                .iter()
                .enumerate()
                .take(i)
                .filter(|f| f.1.field.ident.as_ref().unwrap() == borrowed_field)
                .for_each(|_| {
                    dcx.error(
                        borrowed_field,
                        format!("field `{ident}` borrows `{borrowed_field}`, but drops later",),
                    );
                });
        }
    }

    // Create a lifetime parameter for each field.
    let field_lt_params: Vec<_> = fields
        .iter()
        .filter(|f| f.borrowed.is_some())
        .map(|f| {
            GenericParam::Lifetime(LifetimeParam::new(Lifetime::from_ident(
                f.field.ident.as_ref().unwrap(),
            )))
        })
        .collect();

    let mut generics_with_field_lt = struct_.generics.clone();
    generics_with_field_lt
        .params
        .extend(field_lt_params.iter().cloned());

    let (_, ty_generics_with_field_lt, _) = generics_with_field_lt.split_for_impl();
    let (impl_generics, ty_generics, whr) = struct_.generics.split_for_impl();

    // Wrap each field in a `PhantomInvariant`. For borrowed fields, additionally
    // use `&#lt mut #ty` so the `lt` becomes associated with `#ty` which deduces
    // implied bounds.
    let phantom_fields = fields.iter().map(|f| {
        let ty = &f.field.ty;
        let ident = f.field.ident.as_ref().unwrap();

        if f.borrowed.is_some() {
            let lt = Lifetime::from_ident(ident);
            quote!(
                #ident: ::pin_init::__internal::PhantomInvariant<&#lt mut #ty>,
            )
        } else {
            quote!(
                #[allow(non_snake_case)]
                #ident: ::pin_init::__internal::PhantomInvariant<#ty>,
            )
        }
    });

    let guards = fields.iter().rev().map(|f| {
        let ident = f.field.ident.as_ref().unwrap();
        let span = ident.span();
        if f.borrowed.is_some() {
            quote_spanned!(span =>
                // `LifetimeGuard` implements `Drop` and borrows `#ident`, so Rust must ensure that
                // when the drop impl is called, `#ident` is still alive. Because the guards are
                // generated in reverse field order, this ensures that the lifetimes of fields
                // declared later must strictly outlive the lifetimes of fields declared earlier.
                //
                // For example, in
                // ```
                // struct Foo {
                //     #[borrows(a, b)]
                //     x: &'b &'a (),
                //     a: String,
                //     y: PrintOnDrop<&'b str>,
                //     b: String,
                // }
                // ```
                // it ensures that `b` will strictly outlive `a`.
                //
                // This is needed because implied bounds exist. Rust needs to ensure that types are
                // well-formed; in the above example, `&'b &'a ()` is well-formed only if `a`
                // outlive `b`. To avoid requiring everyone from having to express this bound
                // explicitly when declaring a struct, the `'b: 'a` bound is inferred by the Rust
                // compiler. However this causes an issue, where now `&'a str` can be coerced to
                // `&'b str` because compiler thinks that it shorten the lifetime. This is
                // catastrophical for self-referencing structs, in the above example we'll be able
                // to put a reference to `a` into `y`; but `a` drops first, so when `y` drops, it
                // accesses `a` and causes a use-after-free!
                //
                // The code here essentially reconstruct the dropping order and ask the compiler to
                // check that this won't cause an issue.
                let #ident = ::pin_init::__internal::PhantomInvariant::new();
                // `LifetimeGuard::new` has signature of `(&'a PhantomData<&'a mut T>) ->
                // LifetimeGuard<'a>`. So it serves two purpose: tie the lifetime of binding and the
                // lifetime in the parameter together, and also borrows it.
                let _guard = ::pin_init::__internal::LifetimeGuard::new(&#ident);
            )
        } else {
            quote!(
                // No lifetimes to tie for fields that are not borrowed.
                let #ident = ::pin_init::__internal::PhantomInvariant::new();
            )
        }
    });

    let fields = fields.iter().map(|f| f.field.ident.as_ref().unwrap());

    let struct_span = struct_.ident.span().resolved_at(Span::mixed_site());
    quote_spanned! {struct_span =>
        #[doc(hidden)]
        #[allow(non_snake_case)]
        struct __DropCheck #generics_with_field_lt
            #whr
        {
            #(#phantom_fields)*
        }

        #[allow(non_snake_case)]
        fn __drop_check #impl_generics (
            // This must be present so the function can observe the implied bounds.
            _: &#struct_name #ty_generics,
            f: impl for<#(#field_lt_params,)*>::core::ops::FnOnce(__DropCheck #ty_generics_with_field_lt)
        ) #whr {
            #(#guards)*

            f(__DropCheck {
                #(#fields,)*
            })
        }
    }
}

fn generate_projections(
    vis: &Visibility,
    ident: &Ident,
    generics: &Generics,
    fields: &[FieldInfo<'_>],
) -> TokenStream {
    let pin_lt = Lifetime::new("'__pin", Span::mixed_site());

    let (impl_generics, ty_generics, _) = generics.split_for_impl();
    let mut generics_with_pin_lt = generics.clone();
    generics_with_pin_lt.params.insert(
        0,
        GenericParam::Lifetime(LifetimeParam::new(pin_lt.clone())),
    );
    let (_, ty_generics_with_pin_lt, whr) = generics_with_pin_lt.split_for_impl();

    let field_lt_params: Vec<_> = fields
        .iter()
        .filter(|f| f.borrowed.is_some())
        .map(|f| {
            GenericParam::Lifetime(LifetimeParam::new(Lifetime::from_ident(
                f.field.ident.as_ref().unwrap(),
            )))
        })
        .collect();
    let mut generics_with_field_lt = generics.clone();
    generics_with_field_lt
        .params
        .extend(field_lt_params.iter().cloned());
    let (_, ty_generics_with_field_lt, _) = generics_with_field_lt.split_for_impl();

    let mut generics_with_pin_field_lt = generics_with_field_lt.clone();
    generics_with_pin_field_lt.params.insert(
        0,
        GenericParam::Lifetime(LifetimeParam::new(pin_lt.clone())),
    );
    let (_, ty_generics_with_pin_field_lt, _) = generics_with_pin_field_lt.split_for_impl();

    let this = format_ident!("this");

    let (fields_decl, fields_proj): (Vec<_>, Vec<_>) = fields
        .iter()
        .filter(|f| {
            // Mutably referenced fields cannot be accessed by user at all for aliasing reasons.
            if f.borrowed == Some(Borrowed::Mutable) {
                return false;
            }

            // If the type is not covariant, it must omitted from non-with projection, as such
            // projection shortens the lifetime from fields to '__pin.
            if matches!(f.variance, Some(Variance::NotCovariant(_))) {
                return false;
            }

            true
        })
        .map(|f| {
            let vis = &f.field.vis;
            let ident = f
                .field
                .ident
                .as_ref()
                .expect("only structs with named fields are supported");

            // if `f.ty` contains field lifetimes, which we need to replace them with shorter
            // `'__pin` lifetime as field lifetimes are not available in this context.
            let ty = f.ty.instantiate(&pin_lt);

            // Fields shared-referenced by other fields can only be shared accessed. Fields that
            // references other field and are covariant can also only be given shared reference
            // as mutable reference is invariant.
            let mut_token: Option<Token![mut]> = if f.borrowed.is_none() && f.variance.is_none() {
                Some(Default::default())
            } else {
                None
            };

            let mut accessor = quote!(&#mut_token #this.#ident);
            if f.variance.is_some() {
                accessor = quote!(
                    // SAFETY: we have `SelfRef<..>` which we know is layout compatible with `f.ty`.
                    // Field lifetimes in `f.ty` can be shortened to `#ty` due to covariance (which
                    // is checked later).
                    unsafe { core::mem::transmute::<_, &#mut_token #ty>(#accessor) }
                )
            }

            if f.pinned {
                (
                    quote!(
                        #vis #ident: ::core::pin::Pin<&'__pin #mut_token #ty>,
                    ),
                    quote!(
                        // SAFETY: this field is structurally pinned.
                        #ident: unsafe { ::core::pin::Pin::new_unchecked(#accessor) },
                    ),
                )
            } else {
                (
                    quote!(
                        #vis #ident: &'__pin #mut_token #ty,
                    ),
                    quote!(
                        #ident: #accessor,
                    ),
                )
            }
        })
        .collect();

    let (fields_decl_lt, fields_proj_lt): (Vec<_>, Vec<_>) = fields
        .iter()
        .filter(|f| {
            // Mutably referenced fields cannot be accessed by user at all for aliasing reasons.
            f.borrowed != Some(Borrowed::Mutable)
        })
        .map(|f| {
            let vis = &f.field.vis;
            let ident = f
                .field
                .ident
                .as_ref()
                .expect("only structs with named fields are supported");

            let ty = &f.ty.value;

            // Fields shared-referenced by other fields can only be shared accessed.
            let mut_token: Option<Token![mut]> = if f.borrowed.is_none() {
                Some(Default::default())
            } else {
                None
            };

            let mut accessor = quote!(&#mut_token #this.#ident);
            if f.variance.is_some() {
                accessor = quote!(
                    // SAFETY: we have `SelfRef<..>` which we know is layout compatible with `f.ty`.
                    // We cannot include explicit type name here as the field lifetimes are nameable
                    // in this context.
                    unsafe { core::mem::transmute(#accessor) }
                )
            }

            // In `with_project`, borrowed fields have their field lifetime available, so use it
            // instead of `'__pin`.
            let lt = if f.borrowed.is_some() {
                &Lifetime::from_ident(ident)
            } else {
                &pin_lt
            };

            if f.pinned {
                (
                    quote!(
                        #vis #ident: ::core::pin::Pin<&#lt #mut_token #ty>,
                    ),
                    quote!(
                        // SAFETY: this field is structurally pinned.
                        #ident: unsafe { ::core::pin::Pin::new_unchecked(#accessor) },
                    ),
                )
            } else {
                (
                    quote!(
                        #vis #ident: &#lt #mut_token #ty,
                    ),
                    quote!(
                        #ident: #accessor,
                    ),
                )
            }
        })
        .collect();

    let structurally_pinned_fields_docs: Vec<_> = fields
        .iter()
        .filter(|f| f.pinned)
        .map(|f| format!(" - `{}`", f.field.ident.as_ref().unwrap()))
        .collect();
    let not_structurally_pinned_fields_docs: Vec<_> = fields
        .iter()
        .filter(|f| !f.pinned)
        .map(|f| format!(" - `{}`", f.field.ident.as_ref().unwrap()))
        .collect();
    let docs = format!(" Pin-projections of [`{ident}`]");

    // For fields that references other fields, field access syntax stops working as they're wrapped
    // behind `SelfRef` because their actual lifetime is not on the struct.
    //
    // Generate an accessor method for them.
    let mut accessors = Vec::new();
    for f in fields {
        let ident = f.field.ident.as_ref().unwrap();
        match f.variance {
            // They can be accessed normally, no accessor to be generated.
            None => continue,

            Some(Variance::Covariant(_)) => {
                let f_doc = format!("Access the `{ident}` field on a shared reference of `Self`.");
                let vis = &f.field.vis;

                // Use the span of type for better error message.
                let span = f.ty.value.span().resolved_at(Span::mixed_site());

                let ty = f.ty.instantiate(&pin_lt);

                let long = Lifetime::new("'__long", span);
                let long_ty = f.ty.instantiate(&long);

                let short = Lifetime::new("'__short", span);
                let short_ty = f.ty.instantiate(&short);

                // Add `<'__long: '__short, 'short>` as additional generics.
                let mut covariance_check_generics = generics.clone();
                covariance_check_generics
                    .params
                    .push(GenericParam::Lifetime(LifetimeParam {
                        attrs: Vec::new(),
                        lifetime: long,
                        colon_token: Some(Default::default()),
                        bounds: std::iter::once(short.clone()).collect(),
                    }));
                covariance_check_generics
                    .params
                    .push(GenericParam::Lifetime(LifetimeParam::new(short)));

                accessors.push(quote_spanned!(span =>
                    #[doc = #f_doc]
                    #[inline]
                    #vis fn #ident<#pin_lt>(&#pin_lt self) -> &#pin_lt #ty {
                        // Emit a check to ensure the type is *really* covariant for soundness.
                        fn covariance_check #covariance_check_generics (long: #long_ty) -> #short_ty {
                            long
                        }

                        // SAFETY: `SelfRef` is layout compatible with `#ty` and we have checked
                        // that it is covariant.
                        unsafe { core::mem::transmute(&self.#ident) }
                    }
                ))
            }

            Some(Variance::NotCovariant(_)) => {
                let f_doc = format!("Access the `{ident}` field on a shared reference of `Self`.");
                let vis = &f.field.vis;
                let with_ident = format_ident!("with_{ident}");

                let bound = f.ty.for_bound();
                let ty = &f.ty.value;

                accessors.push(quote!(
                      #[doc = #f_doc]
                      #[inline]
                      #vis fn #with_ident<'__this, R>(&'__this self, f: impl #bound ::core::ops::FnOnce(&'__this #ty) -> R) -> R {
                          // SAFETY: `SelfRef` is layout compatible with `#ty`.
                          f(unsafe { core::mem::transmute(&self.#ident) })
                      }
                  ))
            }
        }
    }

    quote! {
        #[doc = #docs]
        // Allow `non_snake_case` since the same warning will be emitted on
        // the struct definition.
        #[allow(dead_code, non_snake_case)]
        #[doc(hidden)]
        #vis struct __Projection #generics_with_pin_lt
            #whr
        {
            #(#fields_decl)*
            ___pin_phantom_data: ::core::marker::PhantomData<&'__pin #ident #ty_generics>,
        }

        #[doc = #docs]
        // Allow `non_snake_case` since the same warning will be emitted on
        // the struct definition.
        #[allow(dead_code, non_snake_case)]
        #[doc(hidden)]
        #vis struct __ProjectionLt #generics_with_pin_field_lt
            #whr
        {
            #(#fields_decl_lt)*
            ___pin_phantom_data: ::core::marker::PhantomData<&'__pin mut __DropCheck #ty_generics_with_field_lt>,
        }

        impl #impl_generics #ident #ty_generics
            #whr
        {
            /// Pin-projects all fields of `Self`.
            ///
            /// These fields are structurally pinned:
            #(#[doc = #structurally_pinned_fields_docs])*
            ///
            /// These fields are **not** structurally pinned:
            #(#[doc = #not_structurally_pinned_fields_docs])*
            #[inline]
            #vis fn project<'__pin>(
                self: ::core::pin::Pin<&'__pin mut Self>,
            ) -> __Projection #ty_generics_with_pin_lt {
                // SAFETY: we only give access to `&mut` for fields not structurally pinned.
                let #this = unsafe { ::core::pin::Pin::get_unchecked_mut(self) };
                __Projection {
                    #(#fields_proj)*
                    ___pin_phantom_data: ::core::marker::PhantomData,
                }
            }

            /// Pin-projects all fields of `Self` with proper lifetime.
            ///
            /// These fields are structurally pinned:
            #(#[doc = #structurally_pinned_fields_docs])*
            ///
            /// These fields are **not** structurally pinned:
            #(#[doc = #not_structurally_pinned_fields_docs])*
            #[inline]
            #vis fn with_project<'__pin, R>(
                self: ::core::pin::Pin<&'__pin mut Self>,
                f: impl for<#(#field_lt_params,)*>::core::ops::FnOnce(__ProjectionLt #ty_generics_with_pin_field_lt) -> R
            ) -> R {
                // SAFETY: we only give access to `&mut` for fields not structurally pinned.
                let #this = unsafe { ::core::pin::Pin::get_unchecked_mut(self) };
                f(__ProjectionLt {
                    #(#fields_proj_lt)*
                    ___pin_phantom_data: ::core::marker::PhantomData,
                })
            }

            #(#accessors)*
        }
    }
}

fn generate_the_pin_data(
    vis: &Visibility,
    struct_name: &Ident,
    generics: &Generics,
    fields: &[FieldInfo<'_>],
) -> TokenStream {
    let field_lt_params: Vec<_> = fields
        .iter()
        .filter(|f| f.borrowed.is_some())
        .map(|f| {
            GenericParam::Lifetime(LifetimeParam::new(Lifetime::from_ident(
                f.field.ident.as_ref().unwrap(),
            )))
        })
        .collect();

    let mut generics_with_field_lt = generics.clone();
    generics_with_field_lt
        .params
        .extend(field_lt_params.iter().cloned());

    let (impl_generics_with_lt, ty_generics_with_field_lt, _) =
        generics_with_field_lt.split_for_impl();
    let (impl_generics, ty_generics, whr) = generics.split_for_impl();

    let field_accessors = fields
        .iter()
        .map(|f| {
            let vis = &f.field.vis;
            let field_name = f
                .field
                .ident
                .as_ref()
                .expect("only structs with named fields are supported");
            let ty = &f.field.ty;

            let lt = Lifetime::from_ident(field_name);

            let pin_marker = if f.pinned {
                quote!(Pinned)
            } else {
                quote!(Unpinned)
            };

            let (slot_ty, slot_arg) = match f.borrowed {
                None => (quote!(Slot), quote!()),
                Some(Borrowed::Shared) => (
                    // For borrowed fields, create a `SelfRefSlot`, which after initialization
                    // turns into a `SelfRefDropGuard` instead of `DropGuard`.
                    //
                    // They're mostly the same, except that `SelfRefDropGuard` returns `&'field T`
                    // instead of `&'guard T` for let bindings; this allows it to be used to be
                    // used to initialize other fields.
                    //
                    // The soundness of doing so relies on fact that `__make_init` requires a
                    // higher-ranked trait bound on the closure. Within the closure (which is the
                    // caller of the generated slot projection functions here), it can make no
                    // assumptions on the lifetime except for those implied by the struct's bounds,
                    // and we have validated them in `generate_drop_check`.
                    quote!(SelfRefSlot),
                    quote!(#lt, ::pin_init::__internal::Shared, ),
                ),
                Some(Borrowed::Mutable) => (
                    // For borrowed fields, create a `SelfRefSlot`, which after initialization
                    // turns into a `SelfRefDropGuard` instead of `DropGuard`.
                    //
                    // They're mostly the same, except that `SelfRefDropGuard` returns `&'field T`
                    // instead of `&'guard T` for let bindings; this allows it to be used to be
                    // used to initialize other fields.
                    //
                    // The soundness of doing so relies on fact that `__make_init` requires a
                    // higher-ranked trait bound on the closure. Within the closure (which is the
                    // caller of the generated slot projection functions here), it can make no
                    // assumptions on the lifetime except for those implied by the struct's bounds,
                    // and we have validated them in `generate_drop_check`.
                    quote!(SelfRefSlot),
                    quote!(#lt, ::pin_init::__internal::Mutable, ),
                ),
            };

            quote! {
                /// # Safety
                ///
                /// - `slot` is valid and properly aligned.
                /// - `(*slot).#field_name` is properly aligned.
                /// - `(*slot).#field_name` points to uninitialized and exclusively accessed
                ///   memory.
                // Allow `non_snake_case` since the same warning will be emitted on
                // the struct definition.
                #[allow(non_snake_case)]
                #[inline(always)]
                #vis unsafe fn #field_name(
                    self,
                    slot: *mut #struct_name #ty_generics,
                ) -> ::pin_init::__internal::#slot_ty<#slot_arg ::pin_init::__internal::#pin_marker, #ty> {
                    // CAST: `as _` is needed to convert types wrapped inside `SelfRef`.
                    // SAFETY:
                    // - If `#pin_marker` is `Pinned`, the corresponding field is structurally
                    //   pinned.
                    // - Other safety requirements follows the safety requirement.
                    // - If `#slot_ty` is `SelfRefSlot`, the lifetime `#lt` represents that of the
                    //   field.
                    unsafe { ::pin_init::__internal::#slot_ty::new(&raw mut (*slot).#field_name as _) }
                }
            }
        })
        .collect::<TokenStream>();

    quote! {
        // We declare this struct which will host all of the projection function for our type.
        #[doc(hidden)]
        #vis struct __PinDataLt #generics_with_field_lt
            #whr
        {
            // Use `__DropCheck` to capture all lifetimes (including that of borrowed fields) and
            // generics invariantly.
            __phantom: ::core::marker::PhantomData<__DropCheck #ty_generics_with_field_lt>
        }

        impl #impl_generics_with_lt ::core::clone::Clone for __PinDataLt #ty_generics_with_field_lt
            #whr
        {
            fn clone(&self) -> Self { *self }
        }

        impl #impl_generics_with_lt ::core::marker::Copy for __PinDataLt #ty_generics_with_field_lt
            #whr
        {}

        #[allow(dead_code)] // Some functions might never be used and private.
        #[expect(clippy::missing_safety_doc)]
        impl #impl_generics_with_lt __PinDataLt #ty_generics_with_field_lt
            #whr
        {
            #field_accessors
        }

        // Declare a type that serves as the entry point of interaction with the `pin_init!` macro.
        // We use this type instead of defining methods directly on user's type to avoid possibility
        // of name conflicts.
        #[doc(hidden)]
        #vis struct __ThePinData #generics
            #whr
        {
            __phantom: ::pin_init::__internal::PhantomInvariant<#struct_name #ty_generics>,
        }

        impl #impl_generics ::core::clone::Clone for __ThePinData #ty_generics
            #whr
        {
            fn clone(&self) -> Self { *self }
        }

        impl #impl_generics ::core::marker::Copy for __ThePinData #ty_generics
            #whr
        {}

        impl #impl_generics __ThePinData #ty_generics
            #whr
        {
            /// Type inference helper function.
            #[inline(always)]
            #vis fn __make_closure<__F, __E>(self, f: __F) -> __F
            where
                __F: for<#(#field_lt_params,)*>::core::ops::FnOnce(*mut #struct_name #ty_generics, __PinDataLt #ty_generics_with_field_lt) ->
                    ::core::result::Result<::pin_init::__internal::InitOk, __E>,
            {
                f
            }

            #[inline(always)]
            fn __with_lt<#(#field_lt_params,)*>(self) -> __PinDataLt #ty_generics_with_field_lt {
                __PinDataLt { __phantom: ::core::marker::PhantomData }
            }
        }

        // SAFETY: We have added the correct projection functions above to `__ThePinData` and
        // we also use the least restrictive generics possible.
        unsafe impl #impl_generics ::pin_init::__internal::HasPinData for #struct_name #ty_generics
            #whr
        {
            type PinData = __ThePinData #ty_generics;

            unsafe fn __pin_data() -> Self::PinData {
                __ThePinData { __phantom: ::pin_init::__internal::PhantomInvariant::new() }
            }
        }
    }
}

struct SelfReplacer(PathSegment);

impl VisitMut for SelfReplacer {
    fn visit_path_mut(&mut self, i: &mut syn::Path) {
        if i.is_ident("Self") {
            let span = i.span();
            let seg = &self.0;
            *i = parse_quote_spanned!(span=> #seg);
        } else {
            syn::visit_mut::visit_path_mut(self, i);
        }
    }

    fn visit_path_segment_mut(&mut self, seg: &mut PathSegment) {
        if seg.ident == "Self" {
            let span = seg.span();
            let this = &self.0;
            *seg = parse_quote_spanned!(span=> #this);
        } else {
            syn::visit_mut::visit_path_segment_mut(self, seg);
        }
    }

    fn visit_item_mut(&mut self, _: &mut Item) {
        // Do not descend into items, since items reset/change what `Self` refers to.
    }
}
