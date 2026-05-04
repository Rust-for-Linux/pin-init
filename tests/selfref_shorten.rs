use pin_init::*;

#[pin_data]
struct SelfRef {
    #[borrows(foo, bar)]
    part: &'foo str,
    foo: String,
    bar: String,
}

#[test]
fn self_ref() {
    stack_pin_init!(let foo = pin_init!(SelfRef {
        foo: "hello world".to_owned(),
        bar: "hello world".to_owned(),
        part: &foo[..5],
    }));

    stack_pin_init!(let bar = pin_init!(SelfRef {
        foo: "hello world".to_owned(),
        bar: "hello world".to_owned(),
        // In this case, we borrow from a field that lives longers.
        // We're allowed to coerce it into a shorter-living lifetime.
        part: &bar[..5],
    }));
}
