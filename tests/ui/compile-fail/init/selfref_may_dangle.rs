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

// Similar one to the above, but make use of `exists`.
#[pin_data]
struct ExistsOwner<'a>
where
    exists<'x>: 'a,
{
    part: PrintOnDrop<&'outer String>,
    outer: &'x String,
}

fn new<'a>(str: &'a String) -> impl PinInit<SelfRef<'a>, Infallible> {
    pin_init!(SelfRef {
        outer: str,
        part: PrintOnDrop(*outer),
    })
}

fn new_exists<'a>(s: &'a String) -> impl PinInit<ExistsOwner<'a>, Infallible> {
    pin_init!(ExistsOwner {
        outer: s,
        part: PrintOnDrop(*outer),
    })
}

fn main() {
    let str = "hello world".to_owned();
    let _selfref = Box::pin_init(new(&str)).unwrap();
    let _selfref_exists = Box::pin_init(new_exists(&str)).unwrap();
    drop(str);
}
