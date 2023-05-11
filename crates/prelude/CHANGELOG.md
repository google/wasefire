# Changelog

## 0.1.5-git

### Minor

- Add ECDH in `crypto::ec`
- Add SHA-256 in `crypto::hash`
- Add `sync` module for mutex and atomics support
- Add in-place variants for AES-256-GCM
- Implement RustCrypto API for AES-256-GCM

### Patch

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

<!-- Increment to skip CHANGELOG.md test: 3 -->
