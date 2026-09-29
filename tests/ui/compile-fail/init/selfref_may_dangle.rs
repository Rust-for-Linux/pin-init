use pin_init::*;
use std::convert::Infallible;
use std::fmt::Display;

struct PrintOnDrop<T: Display>(T);

impl<T: Display> Drop for PrintOnDrop<T> {
    fn drop(&mut self) {
        println!("Dropping: {}", self.0);
    }
}

#[pin_data]
struct SelfRef<'a> {
    part: PrintOnDrop<&'outer String>,
    outer: &'a String,
}

fn new<'a>(str: &'a String) -> impl PinInit<SelfRef<'a>, Infallible> {
    pin_init!(SelfRef {
        outer: str,
        part: PrintOnDrop(*outer),
    })
}

fn main() {
    let str = "hello world".to_owned();
    let _selfref = Box::pin_init(new(&str)).unwrap();
    drop(str);
}
