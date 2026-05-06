use pin_init::*;

use std::sync::atomic::AtomicU32;

// Test that `?Sized` types can work as expected.
// If macro expansion is not careful it may types into positions that required they're typed.
#[pin_data]
#[repr(C)]
struct ArcInner<T: ?Sized> {
    refcount: AtomicU32,
    data: T,
}
