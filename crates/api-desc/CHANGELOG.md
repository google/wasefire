# Changelog

## 0.2.2-git

### Minor

- Add `crypto::{ecdsa,ecdh,ed25519}` module for ECDSA, ECDH, and Ed25519
- Add `gpio::{Event,register,unregister}` for event support
- Do not pass structs by value in parameters (fix #815)
- Remove `platform::update::metadata()` function
- Add `running` parameter to `platform::version()` function
- Add `platform::running_side()` function for A/B platform update
- Add `clock` module for monotonic wall-time
- Add `crypto::cbc` module for AES-256-CBC
- Add `usb::ctap` module for CTAP HID
- Improve safety documentation
- Add `Api::wasm_markdown()` for top-level documentation
- Use Rust edition 2024
- Use C-string literals to implement dispatch for native applets
- Implement `bytemuck::Pod` for generated `Params`

### Patch

- Update dependencies

## 0.2.1

### Minor

- Remove `debug::exit()` and add `scheduling::exit()` instead

### Patch

- Fix rust and clippy lints
- Update dependencies

## 0.2.0

### Major

- Remove the binary target making this crate a pure library
- Remove implementation of `clap::ValueEnum` for `Lang`

### Minor

- Add `platform::serial()` function
- Change `platform::version()` to allocate the result instead
- Add `store::{keys,clear}()` functions
- Add `platform::protocol` module and `api-platform-protocol` feature for applet RPC

### Patch

- Publish LICENSE file
- Update dependencies
- Use common Wasefire lints
- Use `*const void` instead of `*const u8` for opaque data in `platform::protocol`
- Return a user error instead of an internal error for enums `TryFrom`

## 0.1.6

### Minor

- Add `gpio::last_write()` function
- Add `uart::{start,stop}()` and `uart::set_baudrate()` functions
- Rename `clock` module to `timer`
- Make all API functions return `isize`
- Change `debug::time()` to return `isize` and take `*mut u64`
- Remove unstable `multivalue` support for #355
- Add API features (disabled by default)
- Remove isize conversion functions for enums
- Remove custom error types
- Add `gpio` module
- Add `platform::version()`
- Add `radio::ble` module
- Support array types and all `{i,u}{8,16,32,64}` integers
- Add `platform::update::is_supported()`
- Add `platform::update` module
- Add `platform` module with `platform::reboot()` function
- Call some `env_dispatch` function instead of panicking in `native`
- Prefix function symbols by `env_` with `native` feature
- Implement `bytemuck::Zeroable` for `Results` structs

### Patch

- Clarify conditions when serial events trigger
- Update dependencies
- Fix clippy lint
- Make sure enum values don't skip any value, essentially mapping to `0..N`
- Use `*const void` instead of `*const u8` for opaque data
- Update dependencies
- Fix lints

## 0.1.5

### Minor

- Add `uart` module
- Change return type of `syscall` to unsigned
- Add `store::fragment` module

## 0.1.4

### Minor

- Add `scheduling::abort()`
- Require the `U32` type to implement `bytemuck::Pod`
- Add `debug::time()`, `debug::perf()`, and `debug::Perf` for simple performance
  measurement

### Patch

- Update dependencies
- Add `crypto::gcm::tag_length()` function
- Use `*const u8` instead of `*mut u8` for opaque data

## 0.1.3

### Minor

- Add conversion from enums to `isize`
- Support overwriting API functions with `native`
- Rename `test` feature to `native`
- Add `ec` module with elliptic curve operations
- Add `hash` module with SHA-256, SHA-384, HMAC, and HKDF algorithms
- Support in-place AES-256-GCM by accepting null pointers

### Patch

- Require enum values to be specified (fix #137)

## 0.1.2

### Minor

- Add `is_supported` to `crypto` modules
- Add AES-256-GCM in `crypto::gcm`
- Add return code to `rng::fill_bytes()`
- Add `debug::exit()`

### Patch

- Fix clippy warnings
- Update dependencies
- Test that link names are unique

## 0.1.1

### Minor

- Support wasefire-applet-api test feature
- Add copyright header on generated files

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 5 -->
