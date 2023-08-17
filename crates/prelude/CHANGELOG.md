# Changelog

## 0.3.0-git

### Major

- Make the AES256-GCM tag variable length

### Minor

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

<!-- Increment to skip CHANGELOG.md test: 10 -->
