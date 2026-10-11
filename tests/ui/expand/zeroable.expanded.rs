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
unsafe impl<
    'a,
    T: ::pin_init::Zeroable,
    U: ::pin_init::Zeroable + Trait,
> ::pin_init::Zeroable for WithGenerics<'a, T, U> {}
const _: () = {
    fn assert_zeroable<T: ?::core::marker::Sized + ::pin_init::Zeroable>() {}
    fn ensure_zeroable<'a, T: ::pin_init::Zeroable, U: ::pin_init::Zeroable + Trait>() {
        assert_zeroable::<T>();
        assert_zeroable::<&'a U>();
    }
};
struct WithGenericsMaybe<'a, T, U: Trait> {
    a: T,
    u: &'a U,
}
#[automatically_derived]
unsafe impl<
    'a,
    T: ::pin_init::Zeroable,
    U: ::pin_init::Zeroable + Trait,
> ::pin_init::Zeroable for WithGenericsMaybe<'a, T, U>
where
    T: for<'__dummy> ::pin_init::Zeroable,
    &'a U: for<'__dummy> ::pin_init::Zeroable,
{}
union Union {
    a: usize,
    b: &'static usize,
}
#[automatically_derived]
unsafe impl ::pin_init::Zeroable for Union {}
const _: () = {
    use ::pin_init::__internal::ZeroableCheckFallback as _;
    fn assert_zeroable_any<T: ::pin_init::__internal::IsTrue>(_: T) {}
    fn ensure_zeroable() {
        assert_zeroable_any(
            ::pin_init::__internal::ZeroableCheck::<usize>::new().check()
                | ::pin_init::__internal::ZeroableCheck::<&'static usize>::new().check(),
        );
    }
};
