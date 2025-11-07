# Changelog

## 0.1.4-git

### Patch

- No visible change

## 0.1.3

### Minor

- Use Rust edition 2024

### Patch

- Update dependencies
- Remove dummy alloc dependency
- Fix clippy lints
- Add clippy lint

## 0.1.2

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

<!-- Increment to skip CHANGELOG.md test: 1 -->
