use pin_init::*;

#[pin_data]
struct SelfRef {
    part: &'str str,
    str: String,
    #[borrows(mut mut_str)]
    mut_part: &'mut_str mut str,
    mut_str: String,
}

fn use_self_ref() {
    stack_pin_init!(let foo = pin_init!(SelfRef {
        str: "hello world".to_owned(),
        part: &str[..5],
        mut_str: "hello world".to_owned(),
        mut_part: &mut mut_str[..5],
    }));

    let local = "hello world".to_owned();
    foo.as_mut().with_project(|proj| {
        *proj.part = &local;
    });
}

fn main() {
    use_self_ref();
}
