use pin_init::*;

#[pin_data]
#[derive(serde::Serialize)]
struct Foo {
    #[pin]
    #[serde()]
    #[cfg(test)]
    member: i8,
    #[pin]
    #[serde()]
    #[cfg(not(test))]
    member: u8,
}

#[test]
fn test_attribute() {
    stack_pin_init!(let p = init!(Foo { member: 0 }));
    println!("{}", p.member);
}
