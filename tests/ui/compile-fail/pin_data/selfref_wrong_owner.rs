use pin_init::*;
use std::fmt::Display;

struct PrintOnDrop<T: Display>(T);

impl<T: Display> Drop for PrintOnDrop<T> {
    fn drop(&mut self) {
        println!("Dropping: {}", self.0);
    }
}

#[pin_data]
struct MyStruct {
    borrow: &'owner String,
    owner: String,
    print_on_drop: PrintOnDrop<&'later_owner String>,
    later_owner: String,
}

fn main() {
    stack_pin_init!(let x = pin_init!(MyStruct {
        owner: "hello world".to_owned(),
        borrow: owner,
        later_owner: "hello world".to_owned(),
        print_on_drop: PrintOnDrop(owner),
    }));
}
