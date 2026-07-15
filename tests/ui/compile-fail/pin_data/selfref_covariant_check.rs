use pin_init::*;

#[pin_data]
struct SelfRef {
    not_cov: Box<dyn Fn(&'str str) -> bool + 'str>,
    str: String,
}

fn main() {}
