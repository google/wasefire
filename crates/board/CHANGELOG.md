# Changelog

## 0.6.1-git

### Major

- Add `LastError` trait to check for error after using RustCrypto API.

### Patch

- Update dependencies
- Document all public API

## 0.6.0

### Major

- Add `gpio::Api::last_write()` for stateful output GPIOs
- Add `uart::Api` functions to start, stop, and set the baudrate
- Remove `Unsupported` and `UnsupportedCrypto` implementations
- Change `syscall` return type to `Option<Result<u31, Error>>`
- Add API features (disabled by default)
- Add `Event::Impossible` to always mention the type parameter `B`
- Update `platform::update::Api` to return `Error` instead of `usize`
- Replace `Error` type with `wasefire-error`
- Add `Api::Gpio` for low-level GPIOs
- Add `platform::Api::version()`
- Add `Api::Radio` for radio APIs
- Add `radio::Api::Ble` for Bluetooth Low Energy
- Add `platform::Api::Update` (fix #47)
- Add `Api::Platform` with `platform::Api::reboot()` function
- Require the APIs to be `Send` and the top-level API to be `'static` too

### Minor

- Add `platform::version_helper()` to help implement `platform::Api::version()`
- Add `crypto::Software*` and `debug::Impl` implementations
- Remove unstable `software-crypto` feature

### Patch

- Clarify conditions when serial events trigger
- Fix lints
- Update dependencies
- Use `logger` alias instead of `log` for `wasefire-logger` crate

## 0.5.0

### Major

- Add `Api::Uart` for UARTs

### Minor

- Add `debug::println()` with default implementation
- Add optional `Api::syscall()` method

### Patch

- Move `Support` bound inside the APIs in `crypto`
- Fix software crypto support to be transitive
- Update dependencies

## 0.4.0

### Major

- Add `debug::time()` and `debug::MAX_TIME` for simple performance measurement
- Make the tag length configurable for `crypto::aead::Api`
- Change crypto API to mention `Keysize`, `BlockSize`, and `OutputSize`

### Minor

- Add `UnsupportedCrypto` for partially implemented crypto

### Patch

- Fix clippy lints
- Update dependencies
- Fix `WithSerial` helper to flush all data

## 0.3.0

### Major

- Change API to assume a global state
- Add ECDH and ECDSA support for P256 and P384
- Add the `Types` trait for `Api` associated types
- Add SHA-256, SHA-384, and HMAC support in crypto
- Rename AES-128-CCM and AES-256-GCM types
- Support in-place AES-256-GCM

### Minor

- Add `software-crypto*` features

## 0.2.0

### Major

- Add `Unimplemented` and `Unsupported` interface implementations
- Use composition intead of trait inheritance
- Add `is_supported` to crypto APIs
- Add AES-256-GCM support
- Add `debug::exit()`

### Patch

- Add missing `defmt` dependency for trace and debug levels
- Update dependencies

## 0.1.2

### Patch

- Update wasefire-logger version

## 0.1.1

### Patch

- Build `docs.rs` with `std` feature

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 18 -->
