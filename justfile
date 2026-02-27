check:
    cargo check --all-targets

ui-tests:
    RUSTFLAGS="--cfg UI_TESTS" cargo test

bless-ui-tests:
    TRYBUILD=overwrite MACROTEST=overwrite RUSTFLAGS="--cfg UI_TESTS" cargo test
