// Test that a let binding is generated for each already-initialized field.

use pin_init::*;

#[pin_data]
pub struct Unpinned {
    a: usize,
}

#[test]
fn test_unpinned_binding() {
    stack_pin_init!(let _p = pin_init!(Unpinned {
        a: 1,
        _: {
            println!("{}", *a);
            *a = 2;
            *a = 2;
        }
    }));
}

#[pin_data]
pub struct Pinned {
    #[pin]
    a: usize,
}

#[test]
fn test_pinned_binding() {
    stack_pin_init!(let _p = pin_init!(Pinned {
        a: 1,
        _: {
            println!("{}", *a);
            *a = 2;
            *a = 2;
        }
    }));
}
