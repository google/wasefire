# Changelog

## 0.6.0-git

### Major

- Add non-default `wasm` feature which was the default behavior
- Change `gpio` module to never panic and return an error instead
- Change `button::Listener::new()` to never panic and return an error instead

### Minor

- Add `platform::serial()` to get the platform serial
- Migrate `platform::version()` to new applet API
- Add `rpc::Rpc` trait for `platform::protocol::RpcProtocol`
- Add the `store::{keys,clear}()` functions
- Add the `platform::protocol` module and the `api-platform-protocol` feature for applet RPC
- Allow main to return an error
- Change `led::{get,set}()` to never trap and return an error instead
- Document all public API
- Remove deprecated `println!` macro
- Add `Sha384` and `HmacSha384` to the `rust-crypto` feature

### Patch

- Move lints to `Cargo.toml`, use common Wasefire lints, and make most dependencies optional
- Update dependencies

## 0.5.0

### Major

- Make `uart::Uart` field private
- Rename `clock` module to `timer`
- Change `debug::time()` to return `Result<u64>` instead of `u64`
- Migrate `platform::update` to use `Error` instead of `usize`
- Remove custom error types and use `wasefire-error` instead
- Rename `native` to `test` and use `native` for native applets

### Minor

- Add a deref implementation from `gcm::Key` to `aead::Key`
- Add `gpio::Gpio::{last_write,toggle}()` for stateful output GPIOs
- Add `uart::UartBuilder` and `uart::Uart::new()` to configure a UART
- Add `serial::DelimitedReader` to read delimited frames from serial
- Add `serial::Listener` to listen for serial events
- Migrate to low-level applet API returning `isize`
- Remove experimental `multivalue` support for #355
- Add API features (enabled by default)
- Add `gpio` module
- Add `unsafe-assume-single-core` feature for convenience
- Add `platform::version()`
- Add `radio::ble` module
- Add `platform::update::is_supported()`
- Add `platform::update` module
- Add `platform` module with `platform::reboot()` function
- Use `wasefire-sync` to provide `sync::Mutex`
- Expose the main function as `applet_main` in `native`
- Use the default allocation error handler

### Patch

- Fix lints of nightly-2024-01-14
- Use `ENUM::to_result()` to convert errors
- Only depend on `portable-atomic` through `wasefire-sync`
- Use `wasefire-sync::executed!()` to ensure the allocator is initialized at
  most once
- Build docs.rs for wasm32-unknown-unknown
- Fix broken links in documentation
- Update dependencies

## 0.4.0

### Major

- Update `usb::serial` module to use the new `serial::Serial` interface

### Minor

- Add `serial` module to abstract over serial interfaces
- Add `uart` module for UARTs
- Add `syscall()` for board-specific syscalls
- Add `store::fragment` for fragmented entries in the store

### Patch

- Clean up allocator

## 0.3.0

### Major

- Make the AES256-GCM tag variable length

### Minor

- Add `scheduling::abort()`
- Add `debug::time()` and `debug::perf()` for simple performance measurement
- Add access to SEC1 encoding of ECDSA and ECDH private keys

### Patch

- Update documentation
- Fix clippy lints
- Update dependencies

## 0.2.0

### Major

- Rename `test` feature to `native`
- Rename environment debug flag to `WASEFIRE_DEBUG`

### Minor

- Implement RustCrypto API for SHA, HMAC, and AES-256-GCM
- Add ECDH and ECDSA in `crypto::ec`
- Add SHA-256, SHA-384, HMAC, and HKDF in `crypto::hash`
- Add `sync` module for mutex and atomics support
- Add in-place variants for AES-256-GCM

### Patch

- Fix missing parameter check in `crypto::ccm::decrypt()`
- Fix clippy warnings

## 0.1.4

### Minor

- Add `is_supported` to `crypto` modules
- Add AES-256-GCM in `crypto::gcm`
- Add return code to `rng::fill_bytes()`
- Add `debug::exit()` and `debug::assert*()`

## 0.1.3

### Patch

- Update documentation with 0.1.2 change

## 0.1.2

### Minor

- Add `debug!` macro as a replacement for `println!`
- Deprecate `println!` macro in favor of `debug!`

## 0.1.1

### Minor

- Add test feature
- Add applet!() macro

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 28 -->
