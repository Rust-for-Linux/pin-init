#![cfg(all(unix, not(miri)))]

use std::io::{Error, Result, Write};

use pin_init::*;

#[pin_data]
pub struct Test {}

pub fn init() -> impl PinInit<Test, Error> {
    pin_init!(Test {} ? Error)
}

fn init_fn_ptr<T, E, I: PinInit<T, E>>(_: &I) -> *mut () {
    I::__pinned_init as *mut ()
}

fn read_symbols() -> Result<Vec<String>> {
    let path = std::env::current_exe()?;
    let output = std::process::Command::new("nm")
        .arg(path)
        .arg("--just-symbols")
        .output()?;
    if !output.status.success() {
        Err(Error::other("exit code is not 0"))?
    }
    let symbols = String::from_utf8_lossy(&output.stdout);
    Ok(symbols
        .lines()
        .filter(|x| !x.is_empty())
        .map(|x| x.to_owned())
        .collect())
}

#[test]
fn type_name() {
    let init = init();
    let init_fn = init_fn_ptr(&init);
    std::hint::black_box(init_fn);

    let mut symbols = match read_symbols() {
        Ok(v) => v,
        Err(err) => {
            // This might be some setup issue (e.g. no nm installed).
            // Instead of failing the test, print the error and continue.
            // Do not use `eprintln!()` as it would be captured.
            writeln!(std::io::stderr(), "cannot read symbols {:?}", err).unwrap();
            return;
        }
    };

    // Filter out non-related symbols.
    symbols.retain(|x| x.contains("pin_init") && !x.contains("map"));

    // Find the longest symbol
    symbols.sort_by_key(|x| x.len().wrapping_neg());
    let symbol = &symbols[0];
    println!("{}: {}", symbol.len(), symbol);

    assert!(symbol.len() < 180);
}
