#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::marker::PhantomData;

use pin_init::*;

// Test our ability to handle complex generics.

#[pin_init]
struct Simple<T> {
    t: PhantomData<T>,
}

#[pin_init]
struct Where<T>
where
    T: Clone,
{
    t: PhantomData<T>,
}

#[pin_init]
struct Bound<T: Clone> {
    t: PhantomData<T>,
}

#[pin_init]
struct MoreBounds<'a, 'b: 'a, T> {
    t: PhantomData<&'b &'a T>,
}

#[pin_init]
struct Const<'a, T, const N: usize>
where
    [T; N]: Default,
{
    t: PhantomData<&'a T>,
}

#[pin_init]
struct Defaults<'a, T = usize>
where
    T: Clone,
{
    t: PhantomData<&'a T>,
}

#[pin_init]
struct Unsized<T: ?Sized> {
    t: T,
}

#[pin_init]
struct Tree {
    left: Option<Box<Self>>,
    right: Option<Box<Self>>,
}

#[pin_init]
struct Tree2 where Self: {
    left: Option<Box<Self>>,
    right: Option<Box<Self>>,
}

#[pin_init]
struct Tree3 {
    left: Option<Box<Self>>,
    right: Option<Box<Self>>,
    phantom: [(); {
        struct Foo;

        impl Foo {
            fn new() -> Self {
                Foo
            }
        }

        0
    }]
}
