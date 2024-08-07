# Changelog

## 0.1.2-git

### Minor

- Add `Error::pop()` to propagate errors through space
- Implement `From<std::io::Error>` for `Error` with `std` feature
- Add `std` feature

## 0.1.1

### Minor

- Implement `core::error::Error` for `Error`
- Add `Error::new_const()` and make `Error::{space,code}()` const
- Add an `OutOfBounds` error

### Patch

- Publish LICENSE file
- Depend on `alloc` to silence `unused-crate-dependencies` false positive
- Update dependencies
- Use common Wasefire lints

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 0 -->
