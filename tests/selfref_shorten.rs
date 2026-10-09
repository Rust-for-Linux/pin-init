use std::marker::PhantomData;
use std::sync::Mutex;

use pin_init::*;

#[pin_data]
struct SelfRef {
    #[uses('bar)]
    part: &'foo str,
    foo: String,
    bar: String,
}

#[test]
fn self_ref() {
    stack_pin_init!(let _foo = pin_init!(SelfRef {
        foo: "hello world".to_owned(),
        bar: "hello world".to_owned(),
        part: &foo[..5],
    }));

    stack_pin_init!(let _bar = pin_init!(SelfRef {
        foo: "hello world".to_owned(),
        bar: "hello world".to_owned(),
        // In this case, we borrow from a field that lives longers.
        // We're allowed to coerce it into a shorter-living lifetime.
        part: &bar[..5],
    }));
}

// This struct is covariant over `'a`, despite there are some invariant fields.
#[pin_data]
struct SelfRefCov<'a> {
    cov_part: PhantomData<&'cov_owner ()>,
    cov_owner: &'a String,
    #[uses('owner: invariant)]
    part: Mutex<&'owner str>,
    owner: String,
}

// Swapping the order of unrelated groups should not affect the variance.
#[pin_data]
struct SelfRefCovSwapped<'a> {
    #[uses('owner: invariant)]
    part: Mutex<&'owner str>,
    owner: String,
    cov_part: PhantomData<&'cov_owner ()>,
    cov_owner: &'a String,
}

fn shorten_cov<'long, 'short>(foo: &'short SelfRefCov<'long>) -> &'short SelfRefCov<'short>
where
    'long: 'short,
{
    foo
}

fn shorten_cov_swapped<'long, 'short>(
    foo: &'short SelfRefCovSwapped<'long>,
) -> &'short SelfRefCovSwapped<'short>
where
    'long: 'short,
{
    foo
}

fn init_cov(s: &String) -> impl PinInit<SelfRefCov<'_>> + '_ {
    pin_init!(SelfRefCov {
        cov_part: PhantomData,
        cov_owner: s,
        owner: "early".to_owned(),
        part: Mutex::new("static"),
    })
}

fn init_cov_swapped(s: &String) -> impl PinInit<SelfRefCovSwapped<'_>> + '_ {
    pin_init!(SelfRefCovSwapped {
        owner: "early".to_owned(),
        part: Mutex::new("static"),
        cov_part: PhantomData,
        cov_owner: s,
    })
}

#[test]
fn shorten_with_inv_field() {
    let s = "hello".to_owned();
    stack_pin_init!(let first = init_cov(&s));
    let _first: &SelfRefCov<'_> = shorten_cov(&first);
    stack_pin_init!(let second = init_cov_swapped(&s));
    let _second: &SelfRefCovSwapped<'_> = shorten_cov_swapped(&second);
}

// Shortening is allowed, even for invariant field.
//
// This capability is only allowed in `with_project` though (`with_project_ref` rejects this).
#[pin_data]
struct SelfRefInv {
    #[uses('early: invariant, 'later)]
    part: Mutex<&'early str>,
    early: String,
    later: String,
}

#[test]
fn shorten_inv_with_project() {
    stack_pin_init!(let foo = pin_init!(SelfRefInv {
        early: "early".to_owned(),
        later: "later".to_owned(),
        part: Mutex::new("static"),
    }));
    foo.as_mut().with_project(|proj| {
        *proj.part.get_mut().unwrap() = proj.later.as_str();
    });
    assert_eq!(
        foo.as_ref()
            .with_project_ref(|proj| *proj.part.lock().unwrap()),
        "later"
    );
}

// Mutation in initializer is okay, as long as the order is correct.
// See tests/ui/compile-fail/selfref_mutate_in_init.rs for a wrong example.
#[pin_data]
struct MutateInInit {
    #[uses('early: invariant, 'later)]
    part: Mutex<&'early str>,
    early: String,
    later: String,
}

#[test]
fn mutate_in_init() {
    stack_try_pin_init!(let foo = pin_init!(MutateInInit {
        early: "early".to_owned(),
        later: "later".to_owned(),
        part: Mutex::new(""),
        _: {
            *part.get_mut().unwrap() = later;
        },
    }? ()));
    let foo = foo.unwrap();
    assert_eq!(
        foo.as_ref()
            .with_project_ref(|proj| *proj.part.lock().unwrap()),
        "later"
    );
}
