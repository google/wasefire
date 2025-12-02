# Changelog

## 0.5.0-git

### Major

- Update side table format to support the OpenSK example applet

### Minor

- Implement `Clone` for `Module`
- Extend `Unsupported` with variants specific to WebAssembly 3.0
- Increase supported number of locals from 100 to 1000
- Add `Store::memory()` to access the memory of a given instance

### Patch

- Update dependencies

## 0.4.0

### Major

- Change `Module::new{,_unchecked}()` to expect a module with side-table
- Remove `validate()` in favor of `prepare()`
- Remove `cache` feature in favor of side-table pre-computation
- Remove `Default` implementation for `Module`

### Minor

- Add `prepare()` to validate and prefix the module with a custom section containing the side-table
- Use Rust edition 2024

### Patch

- Fix lints
- Update dependencies

## 0.3.1

### Minor

- Implement `From<Error>` for `wasefire_error::Error`

### Patch

- Fix missing check when module has no data but data count
- Fix rust and clippy lints
- Update dependencies
- Return an error instead of unsupported when too many locals
- Only take the initial frame in `Thread::new()`
- Fix and test the `cache` feature in continuous integration

## 0.3.0

### Major

- Remove unused `Export` type

### Minor

- Add `Store::link_func_custom()` to link functions with custom types
- Expose `ResultType` and `FuncType` for custom types

### Patch

- Publish LICENSE file
- Fix lints
- Support aligned load and store in validation
- Update dependencies
- Refactor parsing of instructions starting with `0xfc`
- Move lints to `Cargo.toml` and use common Wasefire lints

## 0.2.0

### Major

- Change `Error::Unsupported` to take a reason with the `debug` feature

### Minor

- Add `Store::link_func_default()` to generate fake host functions for
  unresolved imported functions returning a single `i32`

### Patch

- Update dependencies
- Update dependencies

## 0.1.4

### Patch

- Re-enable disabled lint due to false positive

## 0.1.3

### Patch

- Update dependencies

## 0.1.2

### Minor

- Use `portable-atomic` to support RV32IMC

### Patch

- Fix new lints
- Fix use of deprecated function
- Update dependencies

## 0.1.1

### Patch

- Panic if `start` calls a host function

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 5 -->
