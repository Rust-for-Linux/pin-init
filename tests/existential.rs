#![allow(dead_code)]

use std::marker::PhantomData;
use std::sync::Mutex;

use pin_init::*;

// An example kernel use case, where `'a` is the lifetime of device, and `T` being some other owned
// data.
struct CoherentBox<'a, T>(PhantomData<(&'a (), T)>);

#[pin_data]
struct Foo<'a>
where
    exists<'x>: 'a,
{
    // We have a mutex to synchronize the access to the data.
    // But we never want to put in a `CoherentBox` from another device,
    // so ideally we want this data structure to be covariant over `'a`.
    #[uses('x: invariant)]
    m: Mutex<CoherentBox<'x, u32>>,
    p: PhantomData<&'a ()>,
}

fn shorten<'long: 'short, 'short>(f: Foo<'long>) -> Foo<'short> {
    f
}

fn use_foo(f: &Foo) {
    f.with_m(|m| {
        let mut g = m.lock().unwrap();
        /* do something with `g` */
        let _ = &mut *g;
    })
}
