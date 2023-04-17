# pin_init

Safe pinned-initialization in Rust.

## The problem

Rust's `Pin` provides sufficient guarantee for C interop and self-referential
structs -- their address are stable once they're pinned and the destructor is
guaranteed to be called before the memory region can be deallocated.

The problem here is "once pinned". `Pin` expects the type can be created without
pinning, and can be pinned later before any operations. This is okay for
`Generator`s, which are created without any self references, and self references
can only be created when polling the generator. For other types, e.g.
`pthread_mutex_t`, it is expected to be pinned from the start.

For demonstration purpose, we will use this type `NeedPin`:
```rust
struct NeedPin {
	// Must points to itself
    address: *const NeedPin,
    _pinned: PhantomPinned,
}

impl NeedPin {
    fn verify(&self) {
        assert!(ptr::eq(self, self.address), "invariant not held");
    }
}

impl Drop for NeedPin {
    fn drop(&mut self) {
        /* Must be called */
    }
}
```

One could separate creating and initialization:
```rust
impl NeedPin {
    unsafe fn uninit() -> Self {
        Self {
            address: ptr::null(),
            _pinned: PhantomPinned,
        }
    }

    unsafe fn init(self: Pin<&mut Self>) -> Result<(), Error> {
        let this = unsafe { self.get_unchecked_mut() };
        this.address = this;
    }
}
```
but this requires unsafe and is very difficult to use.

The ultimate goal is:
1. Safety. We should be able to create and use such pinned type without unsafe.
   (Obviously the pinned type themselves are still unsafe to implement).
2. Ergonomics. The syntax shouldn't be too different from anormal Rust.
3. Aggregatable. A struct containing multiple pinned types can be safely
   created and initialized together.
4. No Implicit Allocation. Allocation should not be required during initialization.
   User should be able to dictate whether it's initialized in a box or on the stack.
5. Fallible. No assumption is made about success of initialization.

## The solution: `pin_init`

This crate provides type `PinUninit` and `InitResult` as the primitives
for safe pinned-initialization. Details about these types can be found in
their respective documentation, but in a nutshell, instead of having a (fallible)
constructor of type `FnOnce() -> Result<T, Err>` like a normal unpinned type,
`pin_init` expect you to present a constructor of type
`for<'a> FnOnce(PinUninit<'a, T>) -> InitResult<'a, T, Err>`.

`NeedPin::new` could be define like this:
```rust
impl NeedPin {
    pub fn new() -> impl Init<Self, Infallible> {
        init_from_closure(unsafe { UnsafeToken::new() }, |mut this: PinUninit<'_, Self>| -> InitResult<'_, Self, Infallible> {
            let v = this.get_mut().as_mut_ptr();
            unsafe { *ptr::addr_of_mut!((*v).address) = v };
            Ok(unsafe { this.init_ok() })
        })
    }
}
```

With Rust's affine type system and borrow checker, the `InitResult` is
essentially a certificate about whether the type is initialized or not.
`NeedPin` can now be easily initialized:
```rust
// In a box
let p: Pin<Box<NeedPin>> = Box::pin_with(NeedPin::new()).unwrap();
// On the stack
init_stack!(p = NeedPin::new());
let p: Pin<&mut NeedPin> = p.unwrap();
```

For structs, if `#[pin_init]` when defining the struct, then
`init_pin!` can create it very similar to the struct expression. Nested
structures are also supported.

```rust
#[pin_init]
struct ManyPin {
    #[pin]
    a: NeedPin,
    b: usize,
}

#[pin_init]
struct TooManyPin {
    #[pin]
    a: NeedPin,
    #[pin]
    b: ManyPin,
}
let p = Box::pin_with(init_pin!(TooManyPin {
    a: NeedPin::new(),
    b: ManyPin {
        a: NeedPin::new(),
        b: 0,
    }),
}));
```

This crate also provides a `UniqueRc` and `UniqueArc`, inspired from servo_arc.
They can be used to mutably initialize `Rc` and `Arc` before they are being shared.
`Rc::pin_with` and `Arc::pin_with` are provided which create `UniqueRc` and `UniqueArc`
internally, pin-initialize it with given constructor, and convert them to the shareable form.

This crate allows safe initialization of pinned data structure.
[`pin-project`](https://github.com/taiki-e/pin-project) can be used to safely access these structs. You can
use both `#[pin_init]` and `#[pin_project]` together with your struct, they even share the same
`#[pin]` field attribute!

See [examples](examples) for some non-artifical examples.
