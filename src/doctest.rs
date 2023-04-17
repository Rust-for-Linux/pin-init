// Helpers for documentation tests.

use pin_init::*;
use std::convert::Infallible;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr;

pub struct NeedPin {
    address: *const NeedPin,
    _pinned: PhantomPinned,
}

impl NeedPin {
    pub fn verify(&self) {
        assert!(ptr::eq(self, self.address), "invariant not held");
    }
}

impl NeedPin {
    pub fn new() -> impl Init<Self, Infallible> {
        init_from_closure(unsafe { UnsafeToken::new() }, |mut this: PinUninit<'_, Self>| {
            let v = this.get_mut().as_mut_ptr();
            unsafe { *ptr::addr_of_mut!((*v).address) = v };
            Ok(unsafe { this.init_ok() })
        })
    }
}
