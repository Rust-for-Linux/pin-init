#![cfg_attr(USE_RUSTC_FEATURES, feature(lint_reasons))]

#[test]
#[cfg_attr(not(UI_TESTS), ignore)]
fn ui_compile_fail() {
    let test_cases = trybuild::TestCases::new();
    test_cases.compile_fail("tests/ui/compile-fail/pinned_drop/*.rs");
    test_cases.compile_fail("tests/ui/compile-fail/pin_data/*.rs");
    test_cases.compile_fail("tests/ui/compile-fail/init/*.rs");
    test_cases.compile_fail("tests/ui/compile-fail/zeroable/*.rs");
}

#[test]
#[cfg_attr(not(UI_TESTS), ignore)]
fn ui_expand() {
    macrotest::expand("tests/ui/expand/*.rs");
}
