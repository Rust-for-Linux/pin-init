use pin_init::*;
use std::fmt::Display;
use std::sync::Mutex;

struct PrintOnDrop<T: Display>(T);

impl<T: Display> Drop for PrintOnDrop<T> {
    fn drop(&mut self) {
        println!("Dropping: {}", self.0);
    }
}

#[pin_data]
struct SelfRef {
    // Given the drop glue, this will essentially imply that `'r` strictly outlive `'r` which is
    // obviously nonsense.
    #[borrows(invariant '_)]
    r: (String, Mutex<PrintOnDrop<&'r str>>),
}

fn main() {
    let mut data = Box::pin_init(pin_init!(SelfRef {
        r: ("hello".to_owned(), Mutex::new(PrintOnDrop("str"))),
    }))
    .unwrap();

    data.as_mut().with_project(|data| {
        *data.r.1.lock().unwrap() = PrintOnDrop(&data.r.0);
    })
}
