use pin_init::*;

#[pin_data]
struct SelfRef {
    part: &'foo str,
    foo: String,
}

fn self_ref(outer: &str) {
    stack_pin_init!(let foo = pin_init!(SelfRef {
        foo: "hello world".to_owned(),
        part: &outer[..5],
    }));
}

fn main() {}
