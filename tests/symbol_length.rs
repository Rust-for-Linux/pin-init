use std::ffi::c_void;

use pin_init::*;

#[pin_data]
pub struct Test {}

pub fn init() -> impl PinInit<Test> {
    pin_init!(Test {})
}

fn init_fn_ptr<T, E, I: PinInit<T, E>>(_: &I) -> *mut c_void {
    I::__pinned_init as _
}

#[test]
fn type_name() {
    let init = init();
    let init_fn = init_fn_ptr(&init);

    let mut symbol = None;
    // Add 1 to the address so it cannot be confused as the end of last function.
    backtrace::resolve(init_fn.wrapping_add(1), |s| {
        symbol = s.name().and_then(|n| n.as_str().map(|s| s.to_owned()));
    });
    let symbol = symbol.unwrap();

    eprintln!("{}: {}", symbol.len(), symbol);
    assert!(symbol.len() < 200);
}
