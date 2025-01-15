# Changelog

## 0.2.1-git

### Minor

- Use Rust edition 2024
- Add `fs::targz_{list,extract}()` to manipulate tarballs
- Add `fs::download()` for download files

### Patch

- Fix clippy lints
- Update dependencies

## 0.2.0

### Major

- Remove `Option` from `action::RustAppletBuild::profile`
- Remove `dir` argument from `action::RustApplet{Build,Test}::run()` (replaced with the `crate_dir`
  field)
- Rename `action::RustAppletBuild::output` to `output_dir`, remove `Option`, and change default
  value from `target/wasefire` to `wasefire`
- Change `action::{Transfer,RustApplet{New,Build,Test}}` to consume `self`
- Remove `Default` for `action::RustAppletBuild` (implement `clap::Parser` instead)
- Add `action` feature to gate the `action` module
- Change API to be async using tokio

### Minor

- Add `action::RustAppletInstall` as a shorthand for `RustAppletBuild` and `AppletInstall`
- Add `action::AppletInstall::wait` to wait for exit status
- Add `action::RustApplet{Build,Test}::crate_dir`
- Add `action::PlatformInfo` to print platform serial and version
- Add `cmd::spawn()` for more control on command execution
- Add `fs::remove_dir_all()` to remove a directory recursively
- Add `fs::rename()` to rename a file
- Add `cargo` and `changelog` modules and features
- Handle more errors during platform discovery and `action::PlatformReboot`
- Extend `fs::write()` first parameter to set the `OpenOptions` too
- Add `error::root_cause_is()` to check the `anyhow::Error` root cause
- Add `action::PlatformLock` for locking a platform protocol
- Expose `action::Transfer` for transfers from host to device
- Add `action::AppletExitStatus` to get the applet exit status
- Add `action::Applet{Install,Uninstall}` for simple applet management
- Add `action::PlatformApiVersion` to get a platform API version
- Change the flags of `action::AppletRpc` to use `action::Wait`
- Add `action::Wait` for commands returning an optional response
- Change the default connection timeout from 1 second to infinite (0 seconds)
- Add `action::PlatformUpdate` for platform update
- Add `action::PlatformList` to list connected platforms
- Add `action::ConnectionOptions` for commands that need a platform connection
- Change the behavior of `fs::copy_if_changed()` to keep an original source

### Patch

- Improve error reporting when executing commands
- Fix incorrect error with UNIX and TCP platform protocols
- Only print commands and file system operations when warnings are logged
- Update dependencies

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 3 -->
