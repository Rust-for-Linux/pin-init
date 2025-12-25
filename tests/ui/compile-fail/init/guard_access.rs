use pin_init::*;

struct Foo {
    x: usize,
}

fn main() {
    let _ = init!(Foo {
        x: 0,
        _: {
            let _ = __x_guard;
        },
    });
}
