use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::Nothing, parse_quote, spanned::Spanned, Block, FnArg, Pat, PatIdent, Result, ReturnType,
    Signature, Stmt, Token, TraitItemFn, Type, TypeImplTrait, TypePath, TypeTraitObject,
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
    let OutputSet {
        raw_recv,
        real_recv,
        recv_span,
        dyn_,
    } = set_output(&mut fun);
    if let Some(default) = fun.default.as_mut() {
        set_body(
            default,
            (raw_recv, real_recv),
            &mut fun.sig,
            dyn_,
            recv_span,
        );
    }
    fun.sig.ident = format_ident!("dyn_{}", fun.sig.ident);
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

struct OutputSet {
    raw_recv: Type,
    real_recv: Type,
    recv_span: Span,
    dyn_: Type,
}

fn set_output(fun: &mut TraitItemFn) -> OutputSet {
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
            let mut args = fun
                .sig
                .inputs
                .iter()
                .map(|arg| match arg {
                    FnArg::Receiver(r) => {
                        let raw = map_receiver(&r.ty);
                        recv = Some((raw.clone(), *r.ty.clone(), r.self_token.span));
                        raw
                    }
                    FnArg::Typed(t) => *t.ty.clone(),
                })
                .collect::<Vec<_>>();
            let (raw_recv, real_recv, recv_span) = recv.unwrap();
            let this = args.remove(0);
            **ty = parse_quote!(::pin_init::DynInit<#dyn_, (#this, #(#args),*)>);
            OutputSet {
                raw_recv,
                real_recv,
                recv_span,
                dyn_,
            }
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
    (raw, _real): (Type, Type),
    sig: &mut Signature,
    dyn_: Type,
    recv_span: Span,
) {
    let args = sig
        .inputs
        .iter_mut()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(arg) => Some(arg),
        })
        .enumerate()
        .map(|(i, arg)| {
            let ident = format_ident!("__arg_{i}");
            arg.pat = Box::new(Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: ident.clone(),
                subpat: None,
            }));
            ident
        })
        .collect::<Vec<_>>();
    let arg_types = sig
        .inputs
        .iter()
        .filter_map(|r| match r {
            FnArg::Receiver(_) => None,
            FnArg::Typed(arg) => Some(arg),
        })
        .map(|arg| &arg.ty);
    let name = &sig.ident;
    let mut this = format_ident!("this");
    this.set_span(recv_span);
    body.stmts.clear();
    body.stmts.push(parse_quote! {
        let __raw_init = |
            slot: *mut (),
            (#this, #(#args),*): (#raw, #(#arg_types),*),
        | -> ::pin_init::InitOk<#dyn_> {
            let #this = unsafe { &*(#this as *const Self) };
            let mut ptr = ::core::ptr::null_mut();
            let value = Self::#name(#this, #(#args),*);
            if false {
                unsafe { *ptr = value };
                ::core::unreachable!()
            } else {
                ptr = slot.cast();
                unsafe { ptr.write(value) };
                let r = unsafe { &*ptr };
                let r = r as &#dyn_;
                ::pin_init::InitOk::from_metadata(::core::ptr::metadata(r))
            }
        };
    });
    body.stmts.push(parse_quote!(let this: *const Self = self;));
    body.stmts.push(parse_quote!(let layout = {
        let mut ptr = ::core::ptr::null_mut();
        if false {
            unsafe { *ptr = Self::#name(self, #(#args),*) };
        }
        fn layout_for<T>(_: *mut T) -> ::core::alloc::Layout {
            ::core::alloc::Layout::new::<T>()
        }
        layout_for(ptr)
    };));
    body.stmts.push(Stmt::Expr(
        parse_quote!(unsafe {
            ::pin_init::DynInit::new(__raw_init, (this.cast::<()>(), #(#args),*), layout)
        }),
        None,
    ));
}
