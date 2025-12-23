use pin_init::*;

#[repr(C, packed)]
struct Foo {
    a: i8,
    b: i32,
    c: i32,
}

fn main() {
    let _ = init!(
        #[disable_initialized_field_access]
        Foo {
            c: -42,
            b: *c,
            a: 0,
        }
    );
}
