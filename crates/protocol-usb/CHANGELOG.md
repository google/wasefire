# Changelog

## 0.2.1-git

### Minor

- Use Rust edition 2024

### Patch

- Fix clippy lints
- Update dependencies

## 0.2.0

### Major

- Implement `wasefire_protocol::Connection` for `Connection` renaming some existing methods

### Minor

- Add `Rpc::enable()` with feature `device` to bypass `HasRpc`
- Migrate to removal of `platform::protocol::Api::disable()`
- Add error message when missing udev rule
- Implement `Debug` for `Candidate<T>` and `Connection<T>`
- Implement `Display` for `Connection<T>`

### Patch

- Fix rust and clippy lints
- Fail to compile if `device` and `host` features are used together
- Update dependencies
- Remove workaround lint false positive

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 2 -->
