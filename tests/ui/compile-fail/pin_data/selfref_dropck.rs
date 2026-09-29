use pin_init::*;

#[pin_data]
struct WrongDropOrder {
    b: u32,
    ptr: &'b u32,
}

struct PrintOnDrop<'a>(&'a str);

impl<'a> Drop for PrintOnDrop<'a> {
    fn drop(&mut self) {
        println!("Dropping: {}", self.0);
    }
}

#[pin_data]
struct UnsoundImplied {
    // Provides an implied bound that `a` outlives `b`!
    ptr: &'b &'a (),
    a: String,
    cannot_refer_a: PrintOnDrop<'b>,
    b: String,
}

fn main() {
    let _foo = Box::pin_init(pin_init!(UnsoundImplied {
        ptr: &&(),
        a: "hello".to_owned(),
        cannot_refer_a: PrintOnDrop(a),
        b: "world".to_owned(),
    }))
    .unwrap();
}
