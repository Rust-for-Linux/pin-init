// SPDX-License-Identifier: Apache-2.0 OR MIT

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    braced, parenthesized,
    parse::{End, Parse},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token, Attribute, Block, Expr, ExprCall, ExprPath, Ident, Index, LitInt, Member, Path, Token,
    Type,
};

use crate::diagnostics::{DiagCtxt, ErrorGuaranteed};

pub(crate) struct Initializer {
    attrs: Vec<InitializerAttribute>,
    this: Option<This>,
    path: Path,
    delim_close_span: Span,
    fields: Punctuated<InitializerField, Token![,]>,
    is_tuple_constructor: bool,
    rest: Option<(Token![..], Expr)>,
    error: Option<(Token![?], Type)>,
}

struct This {
    _and_token: Token![&],
    ident: Ident,
    _in_token: Token![in],
}

struct InitializerField {
    attrs: Vec<Attribute>,
    kind: InitializerKind,
}

enum InitializerKind {
    Value {
        member: Member,
        value: Option<(Token![:], Expr)>,
    },
    Init {
        member: Member,
        _left_arrow_token: Token![<-],
        value: Expr,
    },
    Code {
        _underscore_token: Token![_],
        _colon_token: Token![:],
        block: Block,
    },
}

impl InitializerKind {
    fn member(&self) -> Option<&Member> {
        match self {
            Self::Value { member, .. } | Self::Init { member, .. } => Some(member),
            Self::Code { .. } => None,
        }
    }
}

fn member_ident(member: &Member) -> Ident {
    match member {
        Member::Named(ident) => ident.clone(),
        Member::Unnamed(Index { index, .. }) => format_ident!("_{index}"),
    }
}

fn member_binding(member: &Member) -> Option<Ident> {
    match member {
        Member::Named(ident) => Some(ident.clone()),
        Member::Unnamed(_) => None,
    }
}

enum InitializerAttribute {
    DefaultError(DefaultErrorAttribute),
}

struct DefaultErrorAttribute {
    ty: Box<Type>,
}

pub(crate) fn expand(
    Initializer {
        attrs,
        this,
        path,
        delim_close_span,
        fields,
        is_tuple_constructor,
        rest,
        error,
    }: Initializer,
    default_error: Option<&'static str>,
    pinned: bool,
    dcx: &mut DiagCtxt,
) -> Result<TokenStream, ErrorGuaranteed> {
    if is_tuple_constructor {
        check_tuple_constructor_cfgs(&fields, dcx)?;
    }

    let error = error.map_or_else(
        || {
            if let Some(default_error) = attrs.iter().fold(None, |acc, attr| {
                #[expect(irrefutable_let_patterns)]
                if let InitializerAttribute::DefaultError(DefaultErrorAttribute { ty }) = attr {
                    Some(ty.clone())
                } else {
                    acc
                }
            }) {
                default_error
            } else if let Some(default_error) = default_error {
                syn::parse_str(default_error).unwrap()
            } else {
                dcx.error(delim_close_span, "expected `? <type>` after initializer");
                parse_quote!(::core::convert::Infallible)
            }
        },
        |(_, err)| Box::new(err),
    );
    let slot = format_ident!("slot");
    let (has_data_trait, get_data, init_from_closure) = if pinned {
        (
            format_ident!("HasPinData"),
            format_ident!("__pin_data"),
            format_ident!("pin_init_from_closure"),
        )
    } else {
        (
            format_ident!("HasInitData"),
            format_ident!("__init_data"),
            format_ident!("init_from_closure"),
        )
    };
    let init_kind = get_init_kind(rest, dcx);
    let zeroable_check = match init_kind {
        InitKind::Normal => quote!(),
        InitKind::Zeroing => quote! {
            // The user specified `..Zeroable::zeroed()` at the end of the list of fields.
            // Therefore we check if the struct implements `Zeroable` and then zero the memory.
            // This allows us to also remove the check that all fields are present (since we
            // already set the memory to zero and that is a valid bit pattern).
            fn assert_zeroable<T: ?::core::marker::Sized>(_: *mut T)
            where T: ::pin_init::Zeroable
            {}
            // Ensure that the struct is indeed `Zeroable`.
            assert_zeroable(#slot);
            // SAFETY: The type implements `Zeroable` by the check above.
            unsafe { ::core::ptr::write_bytes(#slot, 0, 1) };
        },
    };
    let this = match this {
        None => quote!(),
        Some(This { ident, .. }) => quote! {
            // Create the `this` so it can be referenced by the user inside of the
            // expressions creating the individual fields.
            let #ident = unsafe { ::core::ptr::NonNull::new_unchecked(slot) };
        },
    };
    // `mixed_site` ensures that the data is not accessible to the user-controlled code.
    let data = Ident::new("__data", Span::mixed_site());
    let init_fields = init_fields(&fields, pinned, &data, &slot);
    let field_check = make_field_check(&fields, init_kind, &path);
    Ok(quote! {{
        // Get the data about fields from the supplied type.
        // SAFETY: TODO
        let #data = unsafe {
            use ::pin_init::__internal::#has_data_trait;
            // Can't use `<#path as #has_data_trait>::#get_data`, since the user is able to omit
            // generics (which need to be present with that syntax).
            #path::#get_data()
        };
        // Ensure that `#data` really is of type `#data` and help with type inference:
        let init = #data.__make_closure::<_, #error>(
            move |slot| {
                #zeroable_check
                #this
                #init_fields
                #field_check
                // SAFETY: we are the `init!` macro that is allowed to call this.
                Ok(unsafe { ::pin_init::__internal::InitOk::new() })
            }
        );
        let init = move |slot| -> ::core::result::Result<(), #error> {
            init(slot).map(|__InitOk| ())
        };
        // SAFETY: TODO
        unsafe { ::pin_init::#init_from_closure::<_, #error>(init) }
    }})
}

enum InitKind {
    Normal,
    Zeroing,
}

fn get_init_kind(rest: Option<(Token![..], Expr)>, dcx: &mut DiagCtxt) -> InitKind {
    let Some((dotdot, expr)) = rest else {
        return InitKind::Normal;
    };
    match &expr {
        Expr::Call(ExprCall { func, args, .. }) if args.is_empty() => match &**func {
            Expr::Path(ExprPath {
                attrs,
                qself: None,
                path:
                    Path {
                        leading_colon: None,
                        segments,
                    },
            }) if attrs.is_empty()
                && segments.len() == 2
                && segments[0].ident == "Zeroable"
                && segments[0].arguments.is_none()
                && segments[1].ident == "init_zeroed"
                && segments[1].arguments.is_none() =>
            {
                return InitKind::Zeroing;
            }
            _ => {}
        },
        _ => {}
    }
    dcx.error(
        dotdot.span().join(expr.span()).unwrap_or(expr.span()),
        "expected nothing or `..Zeroable::init_zeroed()`.",
    );
    InitKind::Normal
}

/// Generate the code that initializes the fields of the struct using the initializers in `field`.
fn init_fields(
    fields: &Punctuated<InitializerField, Token![,]>,
    pinned: bool,
    data: &Ident,
    slot: &Ident,
) -> TokenStream {
    let mut guards = vec![];
    let mut guard_attrs = vec![];
    let mut res = TokenStream::new();
    for InitializerField { attrs, kind } in fields {
        let cfgs = {
            let mut cfgs = attrs.clone();
            cfgs.retain(|attr| attr.path().is_ident("cfg"));
            cfgs
        };

        let member = match kind {
            InitializerKind::Value { member, .. } => member,
            InitializerKind::Init { member, .. } => member,
            InitializerKind::Code { block, .. } => {
                let stmt = &block.stmts;
                res.extend(quote! {
                    #(#attrs)*
                    {
                        #(#stmt)*
                    }
                });
                continue;
            }
        };
        let ident = member_ident(member);

        let slot = if pinned {
            quote! {
                // SAFETY:
                // - `slot` is valid and properly aligned.
                // - `make_field_check` checks that the field is properly aligned.
                // - `make_field_check` prevents the field from being used twice, therefore
                //   it is exclusively accessed and has not been initialized.
                (unsafe { #data.#ident(#slot) })
            }
        } else {
            quote! {
                // For `init!()` macro, everything is unpinned.
                // SAFETY:
                // - The field pointer is valid.
                // - `make_field_check` checks that the field is properly aligned.
                // - `make_field_check` prevents the field from being used twice, therefore
                //   it is exclusively accessed and has not been initialized.
                (unsafe {
                    ::pin_init::__internal::Slot::<::pin_init::__internal::Unpinned, _>::new(
                        &raw mut (*#slot).#member
                    )
                })
            }
        };

        // `mixed_site` ensures that the guard is not accessible to the user-controlled code.
        let guard = format_ident!("__{ident}_guard", span = Span::mixed_site());

        let init = match kind {
            InitializerKind::Value { value, .. } => {
                let value = value
                    .as_ref()
                    .map(|(_, value)| quote!(#value))
                    .unwrap_or_else(|| quote!(#ident));

                quote! {
                    #(#attrs)*
                    let mut #guard = #slot.write(#value);

                }
            }
            InitializerKind::Init { value, .. } => {
                quote! {
                    #(#attrs)*
                    let mut #guard = #slot.init(#value)?;
                }
            }
            InitializerKind::Code { .. } => unreachable!(),
        };
        let binding = member_binding(member).map(|ident| {
            quote! {
                #(#cfgs)*
                // Allow `non_snake_case` since the same warning is going to be reported for the
                // struct field.
                #[allow(unused_variables, non_snake_case)]
                let #ident = #guard.let_binding();
            }
        });

        res.extend(quote! {
            #init

            #binding
        });

        guards.push(guard);
        guard_attrs.push(cfgs);
    }
    quote! {
        #res
        // If execution reaches this point, all fields have been initialized. Therefore we can now
        // dismiss the guards by forgetting them.
        #(
            #(#guard_attrs)*
            ::core::mem::forget(#guards);
        )*
    }
}

/// Generate the check for ensuring that every field has been initialized and aligned.
fn make_field_check(
    fields: &Punctuated<InitializerField, Token![,]>,
    init_kind: InitKind,
    path: &Path,
) -> TokenStream {
    let field_attrs: Vec<_> = fields
        .iter()
        .filter_map(|f| f.kind.member().map(|_| &f.attrs))
        .collect();
    let field_name: Vec<_> = fields.iter().filter_map(|f| f.kind.member()).collect();
    let zeroing_trailer = match init_kind {
        InitKind::Normal => None,
        InitKind::Zeroing => Some(quote! {
            ..::core::mem::zeroed()
        }),
    };
    quote! {
        #[allow(unreachable_code)]
        // We use unreachable code to perform field checks. They're still checked by the compiler.
        // SAFETY: this code is never executed.
        let _ = || unsafe {
            // Create references to ensure that the initialized field is properly aligned.
            // Unaligned fields will cause the compiler to emit E0793. We do not support
            // unaligned fields since `Init::__init` requires an aligned pointer; the call to
            // `ptr::write` for value-initialization case has the same requirement.
            #(
                #(#field_attrs)*
                let _ = &(*slot).#field_name;
            )*

            // If the zeroing trailer is not present, this checks that all fields have been
            // mentioned exactly once. If the zeroing trailer is present, all missing fields will be
            // zeroed, so this checks that all fields have been mentioned at most once. The use of
            // struct initializer will still generate very natural error messages for any misuse.
            ::core::ptr::write(slot, #path {
                #(
                    #(#field_attrs)*
                    #field_name: loop {},
                )*
                #zeroing_trailer
            })
        };
    }
}

type InitFields = Punctuated<InitializerField, Token![,]>;
type InitRest = Option<(Token![..], Expr)>;

fn parse_brace_initializer(
    input: syn::parse::ParseStream<'_>,
) -> syn::Result<(Span, InitFields, InitRest)> {
    let content;
    let brace_token = braced!(content in input);
    let mut fields = Punctuated::new();
    loop {
        let lh = content.lookahead1();
        if lh.peek(End) || lh.peek(Token![..]) {
            break;
        } else if lh.peek(Ident) || lh.peek(LitInt) || lh.peek(Token![_]) || lh.peek(Token![#]) {
            fields.push_value(content.parse()?);
            let lh = content.lookahead1();
            if lh.peek(End) {
                break;
            } else if lh.peek(Token![,]) {
                fields.push_punct(content.parse()?);
            } else {
                return Err(lh.error());
            }
        } else {
            return Err(lh.error());
        }
    }
    let rest = content
        .peek(Token![..])
        .then(|| Ok::<_, syn::Error>((content.parse()?, content.parse()?)))
        .transpose()?;
    Ok((brace_token.span.close(), fields, rest))
}

fn parse_paren_initializer(input: syn::parse::ParseStream<'_>) -> syn::Result<(Span, InitFields)> {
    let content;
    let paren_token = parenthesized!(content in input);
    let mut fields = Punctuated::new();

    while !content.is_empty() {
        let attrs = content.call(Attribute::parse_outer)?;
        if content.peek(Token![<-]) {
            return Err(content.error(
                "`<-` is not supported in tuple constructor syntax; use braces with indices, e.g. `Type { 0 <- init, 1: value }`",
            ));
        }
        let value: Expr = content.parse()?;
        let index = fields.len() as u32;
        fields.push_value(InitializerField {
            attrs,
            kind: InitializerKind::Value {
                member: Member::Unnamed(Index {
                    index,
                    span: value.span(),
                }),
                value: Some((Token![:](value.span()), value)),
            },
        });
        if content.is_empty() {
            break;
        }
        fields.push_punct(content.parse()?);
    }
    Ok((paren_token.span.close(), fields))
}

fn check_tuple_constructor_cfgs(
    fields: &Punctuated<InitializerField, Token![,]>,
    dcx: &mut DiagCtxt,
) -> Result<(), ErrorGuaranteed> {
    for field in fields.iter().take(fields.len().saturating_sub(1)) {
        if let Some(attr) = field.attrs.iter().find(|attr| attr.path().is_ident("cfg")) {
            return Err(dcx.error(
                attr,
                "`#[cfg]` on tuple constructor arguments is only supported on the last argument",
            ));
        }
    }
    Ok(())
}

impl Parse for Initializer {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let this = input.peek(Token![&]).then(|| input.parse()).transpose()?;
        let path = input.parse()?;
        let (delim_close_span, fields, is_tuple_constructor, rest) = if input.peek(token::Brace) {
            let (close_span, fields, rest) = parse_brace_initializer(input)?;
            (close_span, fields, false, rest)
        } else if input.peek(token::Paren) {
            let (close_span, fields) = parse_paren_initializer(input)?;
            (close_span, fields, true, None)
        } else {
            return Err(input.error("expected curly braces or parentheses"));
        };
        let error = input
            .peek(Token![?])
            .then(|| Ok::<_, syn::Error>((input.parse()?, input.parse()?)))
            .transpose()?;
        let attrs = attrs
            .into_iter()
            .map(|a| {
                if a.path().is_ident("default_error") {
                    a.parse_args::<DefaultErrorAttribute>()
                        .map(InitializerAttribute::DefaultError)
                } else {
                    Err(syn::Error::new_spanned(a, "unknown initializer attribute"))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            attrs,
            this,
            path,
            delim_close_span,
            fields,
            is_tuple_constructor,
            rest,
            error,
        })
    }
}

impl Parse for DefaultErrorAttribute {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self { ty: input.parse()? })
    }
}

impl Parse for This {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            _and_token: input.parse()?,
            ident: input.parse()?,
            _in_token: input.parse()?,
        })
    }
}

impl Parse for InitializerField {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        Ok(Self {
            attrs,
            kind: input.parse()?,
        })
    }
}

impl Parse for InitializerKind {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let lh = input.lookahead1();
        if lh.peek(Token![_]) {
            Ok(Self::Code {
                _underscore_token: input.parse()?,
                _colon_token: input.parse()?,
                block: input.parse()?,
            })
        } else if lh.peek(Ident) || lh.peek(LitInt) {
            let member = if lh.peek(Ident) {
                Member::Named(input.parse()?)
            } else {
                let lit: LitInt = input.parse()?;
                let index: u32 = lit.base10_parse().map_err(|_| {
                    syn::Error::new(
                        lit.span(),
                        "tuple field index must be a non-negative integer",
                    )
                })?;
                Member::Unnamed(Index {
                    index,
                    span: lit.span(),
                })
            };
            let lh = input.lookahead1();
            if lh.peek(Token![<-]) {
                Ok(Self::Init {
                    member,
                    _left_arrow_token: input.parse()?,
                    value: input.parse()?,
                })
            } else if lh.peek(Token![:]) {
                Ok(Self::Value {
                    member,
                    value: Some((input.parse()?, input.parse()?)),
                })
            } else if lh.peek(Token![,]) || lh.peek(End) {
                if matches!(member, Member::Unnamed(_)) {
                    Err(lh.error())
                } else {
                    Ok(Self::Value {
                        member,
                        value: None,
                    })
                }
            } else {
                Err(lh.error())
            }
        } else {
            Err(lh.error())
        }
    }
}
