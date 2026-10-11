#![deny(explicit_outlives_requirements)]

use pin_init::*;

#[pin_data]
struct ComplexCapture<'bound> {
    #[uses('gpu: invariant)]
    part: &'gpu (),
    gpu: &'bar1 (),
    #[borrowed]
    bar1: &'bound (),
    bar: &'bound (),
}

fn main() {}
