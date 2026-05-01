use pin_init::*;

#[pin_data]
struct SelfRef {
    #[borrows(foo, bar)]
    part: &'foo str,
    foo: String,
    bar: String,
}

fn self_ref(outer: &str) {
    stack_pin_init!(let foo = pin_init!(SelfRef {
        foo: "hello world".to_owned(),
        bar: "hello world".to_owned(),
        part: &outer[..5],
    }));
}

fn main() {}
