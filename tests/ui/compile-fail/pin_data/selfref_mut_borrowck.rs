use pin_init::*;

#[pin_data]
struct SelfRef {
    part: &'str str,
    #[borrows(mut str)]
    mut_part: &'str mut str,
    str: String,
}

fn use_self_ref() {
    stack_pin_init!(let foo = pin_init!(SelfRef {
        str: "hello world".to_owned(),
        part: &str[..5],
        mut_part: &mut str[..5],
    }));
}

fn main() {
    use_self_ref();
}
