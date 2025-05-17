use pin_init::*;
struct Foo {
    a: usize,
    pub(crate) b: usize,
}
#[automatically_derived]
unsafe impl ::pin_init::Zeroable for Foo {}
const _: () = {
    fn assert_zeroable<T: ?::core::marker::Sized + ::pin_init::Zeroable>() {}
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
unsafe impl ::pin_init::Zeroable for Bar
where
    usize: for<'__dummy> ::pin_init::Zeroable,
    &'static usize: for<'__dummy> ::pin_init::Zeroable,
{}
trait Trait {}
struct WithGenerics<'a, T, U: Trait> {
    a: T,
    u: &'a U,
}
#[automatically_derived]
unsafe impl<'a, T, U: Trait> ::pin_init::Zeroable for WithGenerics<'a, T, U>
where
    T: ::pin_init::Zeroable,
    U: ::pin_init::Zeroable,
{}
const _: () = {
    fn assert_zeroable<T: ?::core::marker::Sized + ::pin_init::Zeroable>() {}
    fn ensure_zeroable<'a, T, U: Trait>()
    where
        T: ::pin_init::Zeroable,
        U: ::pin_init::Zeroable,
    {
        assert_zeroable::<T>();
        assert_zeroable::<&'a U>();
    }
};
