use pin_init::*;

#[pin_data]
struct SelfRef {
    part: &'str str,
    str: String,
}

fn use_self_ref() {
    stack_pin_init!(let foo = pin_init!(SelfRef {
        str: "hello world".to_owned(),
        part: &str[..5],
    }));

    // Should fail due to reference not being mutable.
    *foo.as_mut().project().part = "foo";
}

fn main() {
    use_self_ref();
}
