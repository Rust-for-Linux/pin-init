use pinned_init::*;
struct Foo {
    a: usize,
    pub(crate) b: usize,
}
#[automatically_derived]
unsafe impl ::pinned_init::Zeroable for Foo {}
const _: () = {
    fn assert_zeroable<T: ?::core::marker::Sized + ::pinned_init::Zeroable>() {}
    fn ensure_zeroable() {
        assert_zeroable::<usize>();
        assert_zeroable::<usize>();
    }
};
struct Bar {
    a: usize,
    b: &'static usize,
}
#[automatically_derived]
unsafe impl ::pinned_init::Zeroable for Bar
where
    usize: for<'__dummy> ::pinned_init::Zeroable,
    &'static usize: for<'__dummy> ::pinned_init::Zeroable,
{}
trait Trait {}
struct WithGenerics<'a, T, U: Trait> {
    a: T,
    u: &'a U,
}
#[automatically_derived]
unsafe impl<
    'a,
    T: ::pinned_init::Zeroable,
    U: ::pinned_init::Zeroable + Trait,
> ::pinned_init::Zeroable for WithGenerics<'a, T, U> {}
const _: () = {
    fn assert_zeroable<T: ?::core::marker::Sized + ::pinned_init::Zeroable>() {}
    fn ensure_zeroable<
        'a,
        T: ::pinned_init::Zeroable,
        U: ::pinned_init::Zeroable + Trait,
    >() {
        assert_zeroable::<T>();
        assert_zeroable::<&'a U>();
    }
};
