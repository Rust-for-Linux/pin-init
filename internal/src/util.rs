// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::BTreeSet;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{BoundLifetimes, GenericParam, Generics, Ident, LifetimeParam, Token};
use syn::{Lifetime, Type};

pub(crate) trait TypeExt {
    /// Check if the type includes macro invocations.
    ///
    /// Proc-macros cannot expand macros and peek into them, so if macro is involved sometimes special handling is required.
    fn has_macro(&self) -> bool;

    /// Get the list of unbound lifetimes referenced by the type.
    fn unbound_lifetimes(&self) -> BTreeSet<&Lifetime>;
}

impl TypeExt for Type {
    fn has_macro(&self) -> bool {
        struct HasMacro(bool);

        impl<'ast> Visit<'ast> for HasMacro {
            fn visit_macro(&mut self, _: &'ast syn::Macro) {
                self.0 = true;
            }
        }

        let mut visitor = HasMacro(false);
        visitor.visit_type(self);
        visitor.0
    }

    fn unbound_lifetimes(&self) -> BTreeSet<&Lifetime> {
        struct LifetimeVisitor<'a> {
            bound: BTreeSet<&'a Lifetime>,
        }

        impl<'a> Visit<'a> for LifetimeVisitor<'a> {
            fn visit_lifetime(&mut self, lt: &'a Lifetime) {
                if lt.ident == "static" {
                    return;
                }

                if !self.bound.contains(lt) {
                    self.bound.insert(lt);
                }
            }

            fn visit_trait_bound(&mut self, bound: &'a syn::TraitBound) {
                // In case the type includes a lifetime binder, e.g. `dyn for<'a> Foo`,
                // temporarily remove them from `bound` if they're.

                let mut to_remove = Vec::new();
                if let Some(bound_lt) = &bound.lifetimes {
                    for lt in &bound_lt.lifetimes {
                        let GenericParam::Lifetime(lt) = lt else {
                            continue;
                        };
                        if !self.bound.contains(&&lt.lifetime) {
                            to_remove.push(&lt.lifetime);
                        }
                    }
                }

                self.visit_path(&bound.path);

                for lt in to_remove {
                    self.bound.remove(lt);
                }
            }
        }

        let mut visitor = LifetimeVisitor {
            bound: BTreeSet::new(),
        };
        visitor.visit_type(self);
        visitor.bound
    }
}

pub(crate) struct Binder<T> {
    pub bound: Vec<Lifetime>,
    pub value: T,
}

impl<T> Binder<T> {
    pub(crate) fn new(bound: Vec<Lifetime>, value: T) -> Self {
        Binder { bound, value }
    }

    /// Obtain a `for<...>` that can be used to construct a higher-ranked trait bound.
    pub(crate) fn for_bound(&self) -> BoundLifetimes {
        BoundLifetimes {
            for_token: Default::default(),
            lt_token: Default::default(),
            lifetimes: self
                .bound
                .iter()
                .map(|lt| GenericParam::Lifetime(LifetimeParam::new(lt.clone())))
                .collect(),
            gt_token: Default::default(),
        }
    }

    /// Similar to `for_bound`, but the number of lifetimes is padded to 4.
    pub(crate) fn for_bound_4(&self) -> BoundLifetimes {
        let mut lifetimes: Punctuated<_, _> = self
            .bound
            .iter()
            .map(|lt| GenericParam::Lifetime(LifetimeParam::new(lt.clone())))
            .collect();
        while lifetimes.len() < 4 {
            lifetimes.push(GenericParam::Lifetime(LifetimeParam::new(Lifetime::new(
                &format!("'__lt{}", lifetimes.len()),
                Span::mixed_site(),
            ))));
        }
        BoundLifetimes {
            for_token: Default::default(),
            lt_token: Default::default(),
            lifetimes,
            gt_token: Default::default(),
        }
    }
}

impl Binder<Type> {
    pub(crate) fn instantiate(&self, lifetime: &Lifetime) -> Type {
        // If there's no bound lifetimes, just return.
        if self.bound.is_empty() {
            return self.value.clone();
        }

        // If the type has macro, we cannot peek into it. Use some different approach to replace
        // the type using GAT.
        if self.value.has_macro() {
            let bound = self.for_bound_4();
            let bound_lt = bound.lifetimes.iter();
            let ty = &self.value;
            return syn::Type::Verbatim(quote!(
                <::pin_init::__internal::ForLtImpl<
                    dyn #bound ::pin_init::__internal::WithLt4<#(#bound_lt,)* Of = #ty>
                > as ::pin_init::__internal::ForLt4>::Of<#lifetime, #lifetime, #lifetime, #lifetime>
            ));
        }

        struct LifetimeReplacer<'a> {
            to_replace: BTreeSet<&'a Lifetime>,
            replacement: &'a Lifetime,
        }

        impl VisitMut for LifetimeReplacer<'_> {
            fn visit_lifetime_mut(&mut self, lt: &mut syn::Lifetime) {
                if self.to_replace.contains(lt) {
                    *lt = self.replacement.clone();
                }
            }

            fn visit_trait_bound_mut(&mut self, bound: &mut syn::TraitBound) {
                // In case the type includes a lifetime binder, e.g. `dyn for<'a> Foo`,
                // temporarily remove them from to_replace if they're.

                let mut removed = Vec::new();
                if let Some(bound_lt) = &bound.lifetimes {
                    for lt in &bound_lt.lifetimes {
                        let GenericParam::Lifetime(lt) = lt else {
                            continue;
                        };
                        if let Some(lt) = self.to_replace.take(&lt.lifetime) {
                            removed.push(lt);
                        }
                    }
                }

                self.visit_path_mut(&mut bound.path);

                for lt in removed {
                    self.to_replace.insert(lt);
                }
            }
        }

        let mut ret = self.value.clone();
        LifetimeReplacer {
            to_replace: self.bound.iter().collect(),
            replacement: lifetime,
        }
        .visit_type_mut(&mut ret);
        ret
    }
}

pub(crate) trait LifetimeExt {
    /// Obtain a lifetime from a identifier.
    ///
    /// The created lifetime has the same span.
    fn from_ident(ident: &Ident) -> Self;
}

impl LifetimeExt for Lifetime {
    fn from_ident(ident: &Ident) -> Self {
        Lifetime {
            apostrophe: ident.span(),
            ident: ident.clone(),
        }
    }
}

pub(crate) struct ForGenerics<'a>(&'a Generics);

pub(crate) trait GenericsExt {
    fn for_generics(&self) -> ForGenerics<'_>;
}

impl GenericsExt for Generics {
    fn for_generics(&self) -> ForGenerics<'_> {
        ForGenerics(self)
    }
}

impl ToTokens for ForGenerics<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.0.params.is_empty() {
            return;
        }

        <Token![for]>::default().to_tokens(tokens);
        self.0.split_for_impl().1.to_tokens(tokens);
    }
}
