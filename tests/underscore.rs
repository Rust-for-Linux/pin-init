#![cfg_attr(not(RUSTC_LINT_REASONS_IS_STABLE), feature(lint_reasons))]
use core::cell::Cell;
use pin_init::*;

#[derive(Debug)]
struct Error;

#[pin_data]
#[derive(Debug)]
struct Foo {
    a: usize,
    b: usize,
}

struct Bar;

impl Bar {
    fn new() -> Result<Self, Error> {
        Ok(Self)
    }
    fn make_a(&self) -> usize {
        10
    }
    fn make_b(&self) -> usize {
        20
    }
}

fn error(_bar: &Bar) -> Result<(), Error> {
    Err(Error)
}

fn ok(_bar: &Bar) -> Result<(), Error> {
    Ok(())
}

#[test]
fn delay() {
    let state = &Cell::new(3);
    let init = pin_init!(Foo {
        _: {
            let x = state.get();
            let y = x + 1;
        },
        a: x,
        b: y,
    });
    state.set(42);
    stack_pin_init!(let foo = init);
    assert_eq!(foo.a, 42);
    assert_eq!(foo.b, 43);
}

#[test]
fn error_user() {
    let bar = Bar;
    stack_try_pin_init!(let foo = try_pin_init!(Foo {
        _: { error(&bar)? },
        a: 1,
        b: 2,
    }? Error));

    assert!(foo.is_err());
}

#[test]
fn ok_user() {
    let bar = Bar;
    stack_try_pin_init!(let foo = try_pin_init!(Foo {
        _: { ok(&bar)? },
        a: 1,
        b: 2,
    }? Error));

    assert_eq!(foo.unwrap().a, 1);
}

#[test]
fn split() {
    stack_try_pin_init!(let foo = try_pin_init!(Foo {
        _: { let bar = Bar::new()? },
        a: bar.make_a(),
        b: bar.make_b(),
    }? Error));

    let foo = foo.unwrap();
    assert_eq!(foo.a, 10);
    assert_eq!(foo.b, 20);
}

fn foo() -> bool {
    false
}

fn bar() -> bool {
    true
}

impl Foo {
    pub fn late_error_new() -> impl Init<Self, Error> {
        try_init!(Self {
            _: {
                if foo() {
                    return Err(Error);
                }
            },
            a: 0,
            _: {
                if bar() {
                    return Err(Error);
                }
            },
            b: 0,
        }? Error)
    }
}

#[test]
fn late_error() {
    stack_try_pin_init!(let foo = Foo::late_error_new());
    assert!(foo.is_err());
}
