use std::mem;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::Nothing, parse_quote, spanned::Spanned, visit_mut::VisitMut, Block, FnArg, Pat,
    PatIdent, PatType, Result, ReturnType, Signature, Stmt, Token, TraitItemFn, Type,
    TypeImplTrait, TypePath, TypeTraitObject,
};

pub fn expand(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let e: syn::Expr = parse_quote!(self);
    println!("{e:?}");
    let mut fun: TraitItemFn = syn::parse2(item)?;
    let _: Nothing = syn::parse2(args)?;
    let normal = normal(fun.clone());
    let name = fun.sig.ident.to_string();
    fun.attrs.insert(
        0,
        parse_quote!(#[doc = ::core::concat!("Dynamic version of `", #name, "`.")]),
    );
    fun.sig.ident = format_ident!("dyn_{}", fun.sig.ident);
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
            fun.sig
                .inputs
                .iter()
                .filter_map(|r| match r {
                    FnArg::Receiver(_) => None,
                    FnArg::Typed(arg) => Some(arg),
                })
                .cloned()
                .collect(),
            &mut fun.sig,
            dyn_,
            recv_span,
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
            let args = fun
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
            let error: Type = parse_quote!(::core::convert::Infallible);
            **ty = parse_quote!(::pin_init::DynInit<#dyn_, (#(#args),*)>);
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
    (raw, real): (Type, Type),
    args: Vec<PatType>,
    sig: &mut Signature,
    dyn_: Type,
    recv_span: Span,
) {
    let arg_pats = args.iter().map(|arg| &arg.pat);
    let arg_types = args.iter().map(|arg| &arg.ty);
    let mut this = format_ident!("this");
    this.set_span(recv_span);
    let mut stmts = mem::take(&mut body.stmts);
    for s in &mut stmts {
        SelfReplacer(this.clone()).visit_stmt_mut(s);
    }
    body.stmts.push(parse_quote! {
        unsafe fn __raw_init(
            slot: *mut (),
            (#this, #(#arg_pats),*): (#raw, #(#arg_types),*),
        ) -> <#dyn_ as ::core::ptr::Pointee>::Metadata {
            let value = move || {
                let #this = unsafe { &*(#this as *const Baz) };
                #(#stmts)*
            }();
            let slot = slot.cast();
            unsafe { slot.write(value) };
            let r = unsafe { &*slot };
            let r = r as &#dyn_;
            Ok(::core::ptr::metadata(r))
        }
    });
    body.stmts.push(parse_quote! {
        let this: *const Self = self;
    });
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
        });
    body.stmts.push(Stmt::Expr(
        parse_quote!(unsafe {
            ::pin_init::DynInit::new(__raw_init, (this.cast::<()>(), #(#args),*, layout))
        }),
        None,
    ));
}

struct SelfReplacer(syn::Ident);

impl VisitMut for SelfReplacer {
    fn visit_expr_path_mut(&mut self, i: &mut syn::ExprPath) {
        if i.qself.is_none() && i.path.is_ident("self") {
            i.path = syn::PathSegment {
                ident: self.0.clone(),
                arguments: syn::PathArguments::None,
            }
            .into();
        }
    }

    fn visit_item_mut(&mut self, _: &mut syn::Item) {
        // Do not descend into items, since items reset/change what `Self` refers to.
    }
}
