# Changelog

## 0.3.0-git

### Major

- Remove unused `Export` type

### Minor

- Add `Store::link_func_custom()` to link functions with custom types
- Expose `ResultType` and `FuncType` for custom types

### Patch

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
