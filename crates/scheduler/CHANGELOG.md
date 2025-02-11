# Changelog

## 0.5.0-git

### Major

- Do not allocate if the size is zero for the applet API

### Minor

- Support `usb::ctap` module
- Use Rust edition 2024

### Patch

- Fix clippy lints
- Update dependencies

## 0.4.0

### Major

- Remove `board-api-platform{,-protocol,-update}` features

### Minor

- Gate dead-code when `applet-api-platform-protocol` is disabled
- Exit applets when main exits with no registered callbacks
- Call the `applet::notify_{start,exit}()` hooks
- Trap applets calling into host during init (except for debug printing)
- Support `PlatformLock` protocol call
- Support `AppletExitStatus` protocol call (the platform keeps running when the applet exits)
- Support `Applet{Install,Uninstall}` protocol calls
- Migrate from `debug::exit()` to `scheduling::exit()`
- Support `PlatformUpdate{Metadata,Transfer}` protocol calls

### Patch

- Fix rust and clippy lints
- Reduce logging level of applet trapping (those are not errors)
- Make sure at compile-time that exactly one `native` or `wasm` feature is enabled
- Use `derive-where` instead of `derivative`
- Implement `defmt::Format` for `Key` when `defmt` is enabled
- Stop using `log::Debug2Format()` when logging events
- Make applet optional
- Update dependencies
- Fix missing `build.rs` in cargo package

## 0.3.1

### Minor

- Return an error when calling an unimplemented API
- Support `PlatformVendor` protocol
- Support `platform::serial()` function
- Migrate `platform::version()` to the new board and applet APIs
- Support `store::{keys,clear}()` functions
- Support `platform::protocol` with the `{applet,board}-api-platform-protocol` features
- Change `gpio` module to never trap and return an error instead
- Change `button::{register, unregister}()` to never trap and return an error instead
- Migrate to `Id::new` returning `Result` instead of `Option`
- Migrate to `crypto::{Hash,Hmac}` depending on `crypto::WithError`
- Change `led::{get,set}()` to never trap and return an error instead

### Patch

- Publish LICENSE file
- Introduce `MemoryApi::alloc_copy()` to simplify a common pattern
- Fix lints
- Use common Wasefire lints and make most dependencies optional
- Add `Failure` type to simplify try-blocks for `Scheduler::reply()` arguments
- Update `store` errors mapping
- Use explicit conversion from `Error` to `Trap`
- Simplify `#[cfg(all)]` attributes between board and applet features
- Update dependencies
- Fix HKDF-SHA-384 for outputs longer than 32 bytes

## 0.3.0

### Major

- Do not expose internal details
- Add applet API and board API features (disabled by default)
- Add `native` and `wasm` features (exactly one must be chosen)

### Minor

- Support `gpio::last_write()` function
- Support `uart::{start,stop,set_baudrate}()` functions
- Migrate `clock` module to `timer`
- Remove unstable `software-crypto` feature
- Support applets calling unknown host functions by returning an error
- Migrate to applet API functions always returning `isize`
- Support `debug::time()` new return type
- Migrate to `platform::update` returning `Error`
- Migrate to `wasefire-error`
- Support `gpio` module
- Support `platform::version()`
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

- Fix lints of nightly-2024-01-24
- Support zero-length slices in native
- Fix board API feature gates for AES-128-CCM and AES-256-GCM
- Remove unreachable `multivalue` feature gates
- Correctly gate events of nested APIs
- Rename `debug` internal feature to `internal-debug`
- Fix double-lock issue for native callbacks
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

<!-- Increment to skip CHANGELOG.md test: 3 -->
