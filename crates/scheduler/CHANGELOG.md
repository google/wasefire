# Changelog

## 0.2.1-git

### Minor

- Support `debug::time()` and use it for `debug::println()`
- Add `unsafe-skip-validation` feature

### Patch

- Fix clippy lints
- Fix missing feature forward on dependencies
- Update dependencies

## 0.2.0

### Major

- Change `Scheduler::run()` to take the board as type
- Change `Scheduler::run()` to take the module as argument (fix #132)
- Update `wasefire-board-api` to 0.3.0

### Minor

- Add `const fn Events::new()` for compile-time construction
- Update `wasefire-applet-api` to 0.3.0

## 0.1.2

### Minor

- Add `is_supported()` support in crypto
- Add AES-256-GCM support
- Support `rng::fill_bytes()` return code
- Add `Events::is_empty()`
- Support `debug::exit()`

### Patch

- Fix clippy warnings
- Update dependencies

## 0.1.1

### Patch

- Use `logger::panic!` instead of `core::panic!`

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 15 -->
