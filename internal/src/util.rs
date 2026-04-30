// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::BTreeSet;

use syn::visit::Visit;
use syn::GenericParam;
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
