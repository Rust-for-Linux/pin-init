use pin_init::*;

macro_rules! wrap_init {
    ($($args:tt)*) => {
        ::pin_init::init!(
            $($args)*
        )
    }
}

struct Foo {
    a: u32,
    b: u32,
    c: u32,
}

#[pin_data]
struct SlotName {
    slot: u32,
}

fn main() {
    let c = 3;
    let _ = wrap_init!(Foo {
        a: 1,
        b <- 2,
        c,
    });

    let _ = pin_init!(SlotName { slot: 1 });
}
