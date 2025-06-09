#![cfg_attr(not(RUSTC_LINT_REASONS_IS_STABLE), feature(lint_reasons))]

#[test]
#[cfg_attr(any(miri, NO_UI_TESTS), ignore)]
fn compile_fail() {
    let test_cases = trybuild::TestCases::new();
    test_cases.compile_fail("tests/ui/compile-fail/pinned_drop/*.rs");
    test_cases.compile_fail("tests/ui/compile-fail/pin_data/*.rs");
    test_cases.compile_fail("tests/ui/compile-fail/init/*.rs");
    test_cases.compile_fail("tests/ui/compile-fail/zeroable/*.rs");
}

#[test]
#[cfg_attr(any(miri, NO_UI_TESTS), ignore)]
fn expand() {
    macrotest::expand("tests/ui/expand/*.rs");
}
