# Changelog

## 0.5.0-git

### Major

- Rename `native` to `test` and use `native` for native applets

### Minor

- Add `platform` module with `platform::reboot()` function
- Use `wasefire-mutex` to provide `sync::Mutex`
- Expose the main function as `applet_main` in `native`
- Use the default allocation error handler

### Patch

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

<!-- Increment to skip CHANGELOG.md test: 16 -->
