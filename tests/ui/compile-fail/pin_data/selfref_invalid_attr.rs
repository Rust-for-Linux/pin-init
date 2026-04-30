use pin_init::*;

#[pin_data]
struct InvalidAttr<'a> {
    #[covariant]
    #[not_covariant] // Duplicate variance attribute
    #[borrows(non_exist, mut non_exist_mut)] // Borrows non-existent fields
    explicit: u32,

    implicit: &'non_exist u32,

    bound: &'a u32,
    okay: &'b u32,
    b: u32,
}

fn main() {}
