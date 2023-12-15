# Changelog

## 0.3.0-git

### Major

- Add `native` and `wasm` features (exactly one must be chosen)

### Minor

- Support `radio::ble` module
- Support configuring the memory size with `WASEFIRE_MEMORY_PAGE_COUNT`
- Support `platform::update::is_supported()`
- Support `platform::update`
- Support `platform` and `platform::reboot()`
- Add more debug logging in native mode
- Permit applets to call `debup::println()` during `init()`
- Use `debug::exit()` board API when the applet traps
- Use `log::panic!()` on interpreter errors

### Patch

- Fix parsing of `WASEFIRE_MEMORY_PAGE_COUNT`
- Fix docs.rs build
- Use `wasefire-sync`
- Fix lints
- Use `logger` alias instead of `log` for `wasefire-logger` crate
- Update dependencies

## 0.2.2

### Minor

- Support `debug::println()`
- Support `uart`
- Support `syscall()`
- Support `store::fragment`

### Patch

- Prevent applets to call into unsupported board operations
- Update dependencies

## 0.2.1

### Minor

- Support `scheduling::abort()`
- Support `debug::perf()`
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

<!-- Increment to skip CHANGELOG.md test: 23 -->
