use core::marker::PhantomPinned;
use core::mem::{needs_drop, MaybeUninit};
use core::pin::Pin;

use crate::{
    pin_data, pin_init, pin_init_from_closure, pinned_drop, try_pin_init, uninit, PinInit,
};

#[pin_data(PinnedDrop)]
pub struct PinnedOption<T> {
    present: bool,
    #[pin]
    value: MaybeUninit<T>,
    #[pin]
    _pin: PhantomPinned,
}

#[pinned_drop]
impl<T> PinnedDrop for PinnedOption<T> {
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();
        if needs_drop::<T>() && *this.present {
            let value = unsafe { this.value.get_unchecked_mut() };
            unsafe { value.assume_init_drop() };
        }
    }
}

impl<T> PinnedOption<T> {
    pub fn none() -> impl PinInit<Self> {
        pin_init!(Self {
            present: false,
            value <- uninit::<_, core::convert::Infallible>(),
            _pin: PhantomPinned,
        })
    }

    pub fn some<E>(value: impl PinInit<T, E>) -> impl PinInit<Self, E> {
        try_pin_init!(Self {
            present: true,
            value <- unsafe {pin_init_from_closure(|slot: *mut MaybeUninit<T>| {
                value.__pinned_init(slot.cast())
            })},
            _pin: PhantomPinned,
        }? E)
    }

    pub fn is_some(&self) -> bool {
        self.present
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn as_ref(&self) -> Option<&T> {
        match self.present {
            true => Some(unsafe { self.value.assume_init_ref() }),
            false => None,
        }
    }

    pub fn as_mut(self: Pin<&mut Self>) -> Option<Pin<&mut T>> {
        match self.present {
            true => {
                let value = self.project().value;
                let value = unsafe { value.map_unchecked_mut(|value| value.assume_init_mut()) };
                Some(value)
            }
            false => None,
        }
    }
}
