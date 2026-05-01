// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::BTreeSet;

use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::GenericParam;
use syn::{BoundLifetimes, Ident, LifetimeParam};
use syn::{Lifetime, Type};

pub(crate) trait TypeExt {
    /// Get the list of unbound lifetimes referenced by the type.
    fn unbound_lifetimes(&self) -> BTreeSet<&Lifetime>;
}

impl TypeExt for Type {
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
    #[expect(unused)]
    pub fn for_bound(&self) -> BoundLifetimes {
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
