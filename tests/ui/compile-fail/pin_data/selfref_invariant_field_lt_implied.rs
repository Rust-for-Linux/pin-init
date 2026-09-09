use std::fmt::Display;
use std::marker::PhantomData;
use std::sync::Mutex;

use pin_init::*;

struct PrintOnDrop<T: Display>(T);

impl<T: Display> Drop for PrintOnDrop<T> {
    fn drop(&mut self) {
        println!("Dropping: {}", self.0);
    }
}

#[pin_data]
struct Foo<'a> {
    // In this case, we will establish that `'a: 'early`, but `'early` is invariant, so we need to
    // either make `'a` invariant (which is difficult because we cannot tell in proc macro), or reject
    // such implied bounds.
    marker: PhantomData<&'early &'later ()>,
    #[borrows(invariant 'early)]
    slot: Mutex<PrintOnDrop<&'early str>>,
    early: String,
    later: &'a str,
}

fn shorten<'long, 'short>(foo: &'short Foo<'long>) -> &'short Foo<'short>
where
    'long: 'short,
{
    foo
}

fn main() {
    stack_pin_init!(let foo = pin_init!(Foo {
        early: "early".to_owned(),
        later: "static",
        slot: Mutex::new(PrintOnDrop("initial")),
        marker: PhantomData,
    }));
    {
        // let short = String::from("short");
        // shorten(&foo).with_project_ref(|proj| {
        //     *proj.slot.lock().unwrap() = PrintOnDrop(&short);
        // });
    }
}
