use pin_init::*;

struct Foo {
    a: usize,
}

fn main() {
    let _ = init!(Foo {
        _: {
            return Ok(());
        },
        a: 42,
    });
}
