use pin_init::*;

#[pin_data]
struct InvalidAttr<'a> {
    #[uses('non_exist: covariant)] // Borrows non-existent fields
    explicit: u32,

    implicit: &'non_exist u32,

    #[uses('b: covariant, 'b: invariant)]
    duplicate: u32,

    #[borrowed]
    valid_explicit: u32,

    #[borrowed(mut)]
    valid_explicit_mut: u32,

    #[borrowed = "foobar"]
    #[borrowed(foobar)]
    invalid: u32,

    bound: &'a u32,
    okay: &'b u32,
    b: u32,
}

#[pin_data]
struct Conflict<'a> {
    b: &'a u32,
    #[borrowed]
    a: u32,
}

#[pin_data]
struct InvalidTuple(#[uses(1)] u32, u32);

#[pin_data]
struct InvalidBounds<'a: 'y>
where
    'a: 'x,
{
    y: &'x (),
    x: &'a (),
}

fn main() {}
