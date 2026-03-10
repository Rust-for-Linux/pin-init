use pin_init::*;

struct Foo<T>(T);

fn main() {
    let _ = init!(Foo<()> {
        0 <- (),
    });
}
