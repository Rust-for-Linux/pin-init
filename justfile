# SPDX-License-Identifier: Apache-2.0 OR MIT

check:
    cargo check --all-targets

# -Zthreads=1 is needed to workaround Rust compiler non-determinism.
ui-tests:
    RUSTFLAGS="-Zthreads=1 --cfg UI_TESTS" cargo test

bless-ui-tests:
    TRYBUILD=overwrite MACROTEST=overwrite RUSTFLAGS="-Zthreads=1 --cfg UI_TESTS" cargo test
