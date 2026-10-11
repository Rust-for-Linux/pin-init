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
    #[uses('early: invariant)]
    part: Mutex<PrintOnDrop<&'early str>>,
    early: String,
}

#[pin_data]
struct Indirect {
    #[uses('early: invariant, 'later)]
    part: Mutex<PrintOnDrop<&'early str>>,
    early: String,
    later: String,
}

fn broken_direct() -> impl PinInit<Direct, ()> {
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

fn broken_indirect() -> impl PinInit<Indirect, ()> {
    pin_init!(Indirect {
        part: Mutex::new(PrintOnDrop("")),
        later: "later".to_owned(),
        early: "early".to_owned(),
        _: {
            *part.lock().unwrap() = PrintOnDrop(early);
            if true {
                return Err(());
            }
        },
    }? ())
}

fn main() {
    stack_try_pin_init!(let _s = broken_direct());
    stack_try_pin_init!(let _s = broken_indirect());
}
