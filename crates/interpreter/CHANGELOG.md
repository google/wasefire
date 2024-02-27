# Changelog

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

<!-- Increment to skip CHANGELOG.md test: 4 -->
