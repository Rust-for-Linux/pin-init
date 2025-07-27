use pinned_init::*;

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
