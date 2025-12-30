use pin_init::*;

#[pin_data]
struct Foo {}

fn main() {
    let _ = pin_init!(Foo {
        _: {
            let _ = __data;
        },
    });
}
