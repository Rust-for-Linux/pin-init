#![cfg_attr(feature = "alloc", feature(allocator_api))]
#![cfg_attr(not(RUSTC_LINT_REASONS_IS_STABLE), feature(lint_reasons))]

#[cfg(all(
    feature = "alloc",
    not(miri),
    not(NO_ALLOC_FAIL_TESTS),
    not(target_os = "macos")
))]
#[test]
fn too_big_in_place() {
    #[cfg(feature = "alloc")]
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
