use pin_init::*;

#[derive(Zeroable)]
struct Foo {
    a: usize,
    pub(crate) b: usize,
}

#[derive(MaybeZeroable)]
struct Bar {
    a: usize,
    b: &'static usize,
}

trait Trait {}

#[derive(Zeroable)]
struct WithGenerics<'a, T, U: Trait> {
    a: T,
    u: &'a U,
}

#[derive(MaybeZeroable)]
struct WithGenericsMaybe<'a, T, U: Trait> {
    a: T,
    u: &'a U,
}

#[derive(Zeroable)]
union Union {
    a: usize,
    b: &'static usize,
}
