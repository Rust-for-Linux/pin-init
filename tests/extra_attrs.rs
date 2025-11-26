#![cfg_attr(not(RUSTC_LINT_REASONS_IS_STABLE), feature(lint_reasons))]

use pin_init::*;
use test_dummy_only::Dummy;

#[pin_data]
#[derive(Dummy)]
struct Pointless {
    #[pin]
    #[dummy_attr]
    #[cfg(test)]
    member: i8,
    #[pin]
    #[dummy_attr]
    #[cfg(not(test))]
    member: u8,
}

#[test]
fn multiple_attributes() {
    stack_pin_init!(let p = init!(Pointless { member: 0 }));
    println!("{}", p.member);
}
