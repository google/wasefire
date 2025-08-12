# Changelog

## 0.8.0-git

### Major

- Support changes to `platform` modules for A/B platform update

### Minor

- Support the removal of the `api-radio-ble` feature
- Support the `api-vendor` feature
- Support the `api-fingerprint-{matcher,sensor}` features
- Support the `api-crypto-{ecdsa,ecdh,ed25519}` features
- Support the `api-clock` feature
- Support the `api-crypto-cbc` feature
- Support the `api-usb-ctap` feature
- Improve safety documentation
- Use Rust edition 2024
- Provide `bytemuck::Pod` for `ArrayU32`
- Implement `bytemuck::Pod` for `U32<T>`

### Patch

- Update dependencies

## 0.7.0

### Major

- Update `wasefire-applet-api-macro` version

### Patch

- Make sure at compile-time that exactly one `host` or `wasm` feature is enabled
- Update dependencies

## 0.6.1

### Minor

- Support the `api-platform-protocol` feature

### Patch

- Publish LICENSE file
- Move lints to `Cargo.toml` and use common Wasefire lints
- Update dependencies

## 0.6.0

### Major

- Make all API functions return `isize`
- Add API features (disabled by default)
- Update `wasefire-applet-api-macro` version

### Minor

- Remove unstable `multivalue` support for #355
- Implement conversion from types involving `Error` to `U32`

### Patch

- Build docs.rs for `wasm32-unknown-unknown` only
- Update dependencies
- Fix clippy lint
- Update `native` documentation
- Implement the internal `env_dispatch` function
- Add dependency on `wasefire-logger`

## 0.5.0

### Major

- Update `wasefire-applet-api-macro` version

## 0.4.0

### Major

- Update `wasefire-applet-api-macro` version

### Patch

- Update dependencies

## 0.3.0

### Major

- Rename the `test` feature to `native`
- Update `wasefire-applet-api-macro` version

### Minor

- Add `Curve::int_len()`

## 0.2.0

### Major

- Update `wasefire-applet-api-macro` version

### Patch

- Update Cargo.lock file (it is packaged due to <https://github.com/rust-lang/cargo/issues/11557>
  although examples are not)

## 0.1.2

### Minor

- Update `wasefire-applet-api-macro` version
- Add test feature panicking on applet API calls

## 0.1.1

### Patch

- Build `docs.rs` with `wasm` feature on `thumbv7em-none-eabi` target

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 5 -->
