# Changelog

## 0.1.4-git

### Minor

- Require the `U32` type to implement `bytemuck::Pod`.
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

<!-- Increment to skip CHANGELOG.md test: 7 -->
