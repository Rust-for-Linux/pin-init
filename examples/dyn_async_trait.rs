#![feature(ptr_metadata)]

use pin_init::{dyn_init, DynInPlaceInit};

pub trait Bar {
    fn value(&self) -> i32;
}

pub trait Foo {
    /// Creates a [`Bar`].
    #[dyn_init] // to be bikeshed
    fn bar(&self, arg: i32) -> impl Bar;
}

struct Baz(i32);

impl Bar for Baz {
    fn value(&self) -> i32 {
        self.0
    }
}

impl Foo for Baz {
    #[dyn_init]
    fn bar(&self, arg: i32) -> impl Bar {
        return Baz(self.0 + arg);
    }
}

fn main() {
    let foo: Box<dyn Foo> = Box::new(Baz(21));
    let bar = Box::dyn_init(foo.dyn_bar(21));
    println!("{}", bar.value());
}

/**
```
// desugars to:


pub trait Foo {
    fn foo(&self, arg: i32) -> DynInit<dyn Bar, (*const (), i32), Infallible>;
}

struct Baz(i32);

impl Foo for Baz {
    fn foo(&self, arg: i32) -> DynInit<dyn Bar, (*const (), i32), Infallible> {
        fn raw_init(slot: *mut (), (this, arg): (*const (), i32)) -> Result<<dyn Bar>::Metadata, Infallible> {
            let init = move || -> impl Init<_, Infallible> {
                let this = unsafe { &*(this as *const Baz) };
                Baz(this.0 + arg)
            }();
            let slot = slot.cast();
            unsafe { init.__init(slot)? };
            let r = unsafe { &*slot };
            let r = r as &dyn Bar;
            Ok(core::ptr::metadata(r))
        }
        let this: *const Self = self;
        DynInit::new(raw_init, (this.cast::<()>(), arg))
    }
}
```
*/
const _: () = ();
