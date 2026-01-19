#![cfg_attr(feature = "alloc", feature(allocator_api))]
#![cfg_attr(USE_RUSTC_FEATURES, feature(lint_reasons))]

#[test]
#[cfg(feature = "alloc")]
#[cfg_attr(any(miri, NO_ALLOC_FAIL_TESTS, target_os = "macos"), ignore)]
fn too_big_in_place() {
    use core::alloc::AllocError;

    use pin_init::*;
    use std::sync::Arc;

    // should be too big with current hardware.
    assert!(matches!(
        Box::init(init_zeroed::<[u8; 1024 * 1024 * 1024 * 1024]>()),
        Err(AllocError)
    ));
    // should be too big with current hardware.
    assert!(matches!(
        Arc::init(init_zeroed::<[u8; 1024 * 1024 * 1024 * 1024]>()),
        Err(AllocError)
    ));
}
