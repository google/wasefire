# Changelog

## 0.6.0-git

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

- Update Cargo.lock file (it is packaged due to
  https://github.com/rust-lang/cargo/issues/11557 although examples are not)

## 0.1.2

### Minor

- Update `wasefire-applet-api-macro` version
- Add test feature panicking on applet API calls

## 0.1.1

### Patch

- Build `docs.rs` with `wasm` feature on `thumbv7em-none-eabi` target

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 8 -->
