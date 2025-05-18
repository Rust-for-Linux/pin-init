use std::mem;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::Nothing, parse_quote, spanned::Spanned, Block, FnArg, PatType, Result, ReturnType, Stmt,
    Token, TraitItemFn, Type, TypeImplTrait, TypePath, TypeTraitObject,
};

pub fn expand(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let mut fun: TraitItemFn = syn::parse2(item)?;
    let _: Nothing = syn::parse2(args)?;
    let normal = normal(fun.clone());
    let name = fun.sig.ident.to_string();
    fun.attrs.insert(
        0,
        parse_quote!(#[doc = ::core::concat!("Dynamic version of `", #name, "`.")]),
    );
    fun.sig.ident = format_ident!("dyn_{}", fun.sig.ident);
    let (raw, real) = set_output(&mut fun);
    if let Some(default) = fun.default.as_mut() {
        set_body(
            default,
            (raw, real),
            fun.sig
                .inputs
                .iter()
                .filter_map(|r| match r {
                    FnArg::Receiver(_) => None,
                    FnArg::Typed(arg) => Some(arg),
                })
                .cloned()
                .collect(),
        );
    }
    Ok(quote! {
        #normal
        #fun
    })
}

fn normal(mut fun: TraitItemFn) -> TraitItemFn {
    fun.sig
        .generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(Self: ::core::marker::Sized));
    fun
}

/// sets the output type of the function to `DynInit` & returns both the thin & concrete types of
/// the receiver
fn set_output(fun: &mut TraitItemFn) -> (Type, Type) {
    if fun.sig.asyncness.take().is_some() {
        let (arrow, ty) = match &fun.sig.output {
            ReturnType::Default => (Token![->](fun.sig.span()), parse_quote!(())),
            ReturnType::Type(arrow, ty) => (*arrow, ty.clone()),
        };
        fun.sig.output = parse_quote!(#arrow impl Future<Output = #ty>);
    }
    match &mut fun.sig.output {
        ReturnType::Default => todo!("expected `-> impl Trait`"),
        ReturnType::Type(_, ty) => {
            let dyn_ = match &**ty {
                Type::ImplTrait(TypeImplTrait { bounds, impl_token }) => {
                    Type::TraitObject(TypeTraitObject {
                        bounds: bounds.clone(),
                        dyn_token: Some(Token![dyn](impl_token.span)),
                    })
                }
                _ => todo!("expected `-> impl Trait`"),
            };
            let mut recv = None;
            let args = fun
                .sig
                .inputs
                .iter()
                .map(|arg| match arg {
                    FnArg::Receiver(r) => {
                        let raw = map_receiver(&r.ty);
                        recv = Some((raw.clone(), *r.ty.clone()));
                        raw
                    }
                    FnArg::Typed(t) => *t.ty.clone(),
                })
                .collect::<Vec<_>>();
            let (raw, real) = recv.unwrap();
            let error: Type = parse_quote!(::core::convert::Infallible);
            **ty = parse_quote!(::pin_init::DynInit<#dyn_, (#(#args),*), #error>);
            (raw, real)
        }
    }
}

fn map_receiver(r: &Type) -> Type {
    match r {
        Type::Reference(r) if is_self_ty(&r.elem) => match r.mutability {
            None => parse_quote!(*const ()),
            Some(_) => parse_quote!(*mut ()),
        },
        _ => todo!("{r:?}"),
    }
}

fn is_self_ty(ty: &Type) -> bool {
    match ty {
        Type::Path(TypePath { qself: None, path }) => path.is_ident("Self"),
        _ => false,
    }
}

fn set_body(
    body: &mut Block,
    (raw, real): (Type, Type),
    args: Vec<PatType>,
    error: Type,
    dyn_: Type,
) {
    let arg_pats = args.iter().map(|arg| &arg.pat);
    let arg_types = args.iter().map(|arg| &arg.ty);
    let stmts = mem::take(&mut body.stmts);
    body.stmts.push(parse_quote! {
        fn __raw_init(
            slot: *mut (),
            (this, #(#arg_pats),*): (#raw, #(#arg_types),*),
        ) -> ::core::result::Result<<#dyn_ as ::core::ptr::Pointee>::Metadata, #error> {
            let init = move || -> impl ::pin_init::Init<_, #error> {
                let this = unsafe { &*(this as *const Baz) };
                #(#stmts)*
            }();
            let slot = slot.cast();
            unsafe { ::pin_init::Init::__init(init, slot)? };
            let r = unsafe { &*slot };
            let r = r as &#dyn_;
            Ok(::core::ptr::metadata(r))
        }
    });
    body.stmts.push(parse_quote! {
        let this: *const Self = self;
    });
    body.stmts.push(parse_quote! {
        DynInit::new(raw_init, (this.cast::<()>(), arg))
    });
}
