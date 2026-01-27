# Changelog

## 0.1.3-git

### Minor

- Change the schema of `str` from builtin to `[u8]`
- Implement `Wire` for `alloc::string::String`
- Implement `Wire` for `alloc::borrow::Cow`
- Remove useless quantification on `Yoke::{try_,}map()` function argument
- Optimize memory usage when encoding slices in particular of small elements like `Vec<u8>`

### Patch

- Factorize recursive schema logic
- Update dependencies

## 0.1.2

### Minor

- Implement `Wire` for `wasefire_common::Side`
- Use Rust edition 2024

### Patch

- Allow `uninlined-format-args` clippy lint
- Fix clippy lints
- Update dependencies

## 0.1.1

### Minor

- Implement `Debug` for `Yoke`

### Patch

- Fix rust and clippy lints
- Update dependencies

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 6 -->
