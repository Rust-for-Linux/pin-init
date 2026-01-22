# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `[pin_]init_scope` functions to run arbitrary code inside of an initializer.
- `&'static mut MaybeUninit<T>` now implements `InPlaceWrite`. This enables users to use external
  allocation mechanisms such as `static_cell`.

### Changed

- `#[pin_data]` now generates a `*Projection` struct similar to the `pin-project` crate.
- Add initializer code blocks to `[try_][pin_]init!` macros: make initializer
  macros accept any number of `_: {/* arbitrary code */},` & make them run the
  code at that point.
- Make the `[try_][pin_]init!` macros expose initialized fields via a `let`
  binding as `&mut T` or `Pin<&mut T>` for later fields.
- Rewrote all proc-macros (`[pin_]init!`, `#[pin_data]`, `#[pinned_drop]`,
  `derive([Maybe]Zeroable)`),  using `syn` with better diagnostics.
- `derive([Maybe]Zeroable)` now support tuple structs.
- `[pin_]init!` now supports attributes on fields (such as `#[cfg(...)]`).
- Add a `#[default_error(<type>)]` attribute to `[pin_]init!` to override the
  default error (when no `? Error` is specified).
- Support packed struct in `[pin_]init!` with `#[disable_initialized_field_access]`.

### Removed

- `try_[pin_]init!` have been removed in favor of merging their feature with
  `[pin_]init!`.

### Fixed

- Corrected `T: Sized` bounds to `T: ?Sized` in the generated `PinnedDrop`
  check by `#[pin_data]`.

## [0.0.10] - 2025-08-19

### Added

- `Wrapper<T>` trait added for creating wrapper structs with a structurally pinned value.
- `MaybeZeroable` derive macro to try to derive `Zeroable`, but not error if not all fields
  implement it.
- `unsafe fn cast_[pin_]init()` functions to unsafely change the initialized type of an initializer
- `impl<T, E> [Pin]Init<T, E> for Result<T, E>`, so results are now (pin-)initializers
- add `Zeroable::init_zeroed()` delegating to `init_zeroed()`
- add new `zeroed()`, a safe version of `mem::zeroed()` and also provide it via `Zeroable::zeroed()`
- implement `Zeroable` for `Option<&T>` and `Option<&mut T>`
- implement `Zeroable` for `Option<[unsafe] [extern "abi"] fn(...args...) -> ret>` for `"Rust"` and
  `"C"` ABIs and up to 20 arguments

### Changed

- `InPlaceInit` now only exists when the `alloc` or `std` features are enabled
- added support for visibility in `Zeroable` derive macro
- added support for `union`s in `Zeroable` derive macro
- renamed the crate from `pinned-init` to `pin-init` and `pinned-init-macro` to `pin-init-internal`
- blanket impls of `Init` and `PinInit` from `impl<T, E> [Pin]Init<T, E> for T` to
  `impl<T> [Pin]Init<T> for T`
- renamed `zeroed()` to `init_zeroed()`

### Fixed

- `Zeroable` implementation for `Option<Box<T>>` & `Option<NonNull<T>>` to only allow `T: Sized`
  (soundness issue)

## [0.0.9] - 2024-12-02

### Added

- `InPlaceWrite` trait to re-initialize already existing allocations,
- `assert_pinned!` macro to check if a field is marked with `#[pin]`,
- compatibility with stable Rust, thanks a lot to @bonzini! #24 and #23:
  - the `alloc` feature enables support for `allocator_api` and reflects the old behavior, if it is
    disabled, then infallible allocations are assumed (just like the standard library does).

### Fixed

- guard hygiene wrt constants in `[try_][pin_]init!`

## [0.0.8] - 2024-07-07

### Changed

- return type of `zeroed()` from `impl Init<T, E>` to `impl Init<T>` (also removing the generic
  parameter `E`)
- removed the default error of `try_[pin_]init!`, now you always have to specify an error using
  `? Error` at the end
- put `InPlaceInit` behind the `alloc` feature flag, this allows stable usage of the `#![no_std]`
  part of the crate

## [0.0.7] - 2024-04-09

### Added

- `Zeroable` derive macro
- `..Zeroable::zeroed()` tail expression support in `[try_][pin_]init!` macros: allowed to omit
  fields, omitted fields are initialized with `0`
- `[pin_]chain` functions to modify a value after an initializer has run
- `[pin_]init_array_from_fn` to create `impl [Pin]Init<[T; N], E>` from a generator closure
  `fn(usize) -> impl [Pin]Init<T, E>`
- `impl Zeroable for UnsafeCell`

### Changed

- `PinInit` is now a supertrait of `Init` (before there was a blanket impl)

### Removed

- coverage workflow and usage of `#[feature(no_coverage)]`
- `impl Zeroable for Infallible` (see [Security](#security))

### Fixed

- `Self` in generic bounds on structs with `#[pin_data]`
- const generic default parameter values can now be used on structs with `#[pin_data]`

### Security

- `impl Zeroable for Infallible` (#13) it was possible to trigger UB by creating a value of type
  `Box<Infallible>` via `Box::init(zeroed())`

## [0.0.6] - 2023-04-08

[unreleased]: https://github.com/Rust-for-Linux/pin-init/compare/v0.0.10...HEAD
[0.0.10]: https://github.com/Rust-for-Linux/pin-init/compare/v0.0.9...v0.0.10
[0.0.9]: https://github.com/Rust-for-Linux/pin-init/compare/v0.0.8...v0.0.9
[0.0.8]: https://github.com/Rust-for-Linux/pin-init/compare/v0.0.7...v0.0.8
[0.0.7]: https://github.com/Rust-for-Linux/pin-init/compare/v0.0.6...v0.0.7
[0.0.6]: https://github.com/Rust-for-Linux/pin-init/releases/tag/v0.0.6
