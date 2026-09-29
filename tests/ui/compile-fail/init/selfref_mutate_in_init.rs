#![allow(warnings)]
use pin_init::*;
use std::fmt::Display;
use std::marker::PhantomData;
use std::sync::Mutex;

struct PrintOnDrop<T: Display>(T);

impl<T: Display> Drop for PrintOnDrop<T> {
    fn drop(&mut self) {
        println!("Dropping: {}", self.0);
    }
}

#[pin_data]
struct Direct {
    #[borrows(invariant 'early)]
    part: Mutex<PrintOnDrop<&'early str>>,
    early: String,
}

fn broken() -> impl PinInit<Direct, ()> {
    pin_init!(Direct {
        part: Mutex::new(PrintOnDrop("")),
        early: "hello world".to_owned(),
        _: {
            *part.lock().unwrap() = PrintOnDrop(early);
            if true {
                return Err(());
            }
        },
    }? ())
}

fn main() {
    stack_try_pin_init!(let _s = broken());
}
