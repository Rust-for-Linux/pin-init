use core::marker::PhantomData;
use pin_init::*;

struct Queue<T: Iterator<Item = u8> + 'static> {
    _marker: PhantomData<T>,
}

impl<T: Iterator<Item = u8> + 'static> Queue<T> {
    fn new() -> impl PinInit<Self> {
        init!(Self {
            _marker: PhantomData,
        })
    }
}

#[pin_data]
struct InlineBoundHandler<T: Iterator<Item = u8> + 'static> {
    #[pin]
    queue: Queue<T>,
}

impl<T: Iterator<Item = u8> + 'static> InlineBoundHandler<T> {
    fn new() -> impl PinInit<Self> {
        pin_init!(Self {
            queue <- Queue::new(),
        })
    }
}

#[pin_data]
struct WhereBoundHandler<T>
where
    T: Iterator<Item = u8> + 'static,
{
    #[pin]
    queue: Queue<T>,
}

impl<T> WhereBoundHandler<T>
where
    T: Iterator<Item = u8> + 'static,
{
    fn new() -> impl PinInit<Self> {
        pin_init!(Self {
            queue <- Queue::new(),
        })
    }
}

#[test]
fn pin_data_preserves_inline_and_where_bounds() {
    type Iter = core::iter::Empty<u8>;

    stack_pin_init!(let inline = InlineBoundHandler::<Iter>::new());
    let inline_proj = inline.as_mut().project();
    let _ = inline_proj.queue;

    stack_pin_init!(let where_ = WhereBoundHandler::<Iter>::new());
    let where_proj = where_.as_mut().project();
    let _ = where_proj.queue;
}
