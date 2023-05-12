# Changelog

## 0.3.0-git

### Major

- Add P256 and P384 support in crypto
- Add the `Types` trait for `Api` associated types
- Add SHA-256 and HMAC-SHA-256 support in crypto
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

<!-- Increment to skip CHANGELOG.md test: 4 -->
