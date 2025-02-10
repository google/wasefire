# Changelog

## 0.2.1-git

### Minor

- Use Rust edition 2024
- Add `self-update` to download and install the latest CLI

### Patch

- Update dependencies

## 0.2.0

### Major

- Rename `--output` to `--output-dir` for `rust-applet-build`
- Rename `--serial` to `--protocol` with more support
- Move `--serial` and `--timeout` to commands that need them

### Minor

- Add `rust-applet-install` to build and install an applet
- Add `--crate-dir` for `rust-applet-{build,test}`
- Add `platform-info` to print platform serial and version
- Add `host` to start a host platform
- Support `RUST_LOG` to control logging
- Add `platform-lock` to lock a platform protocol
- Add `applet-exit-status` to get an applet exit status
- Implement `applet-{install,uninstall}` for applet management
- Add `platform-update-{metadata,transfer}` for platform update

### Patch

- Print `wasefire` instead of `wasefire-cli` with `--version`
- Check for bad command-line configuration
- Update dependencies
- Restore release builds to the default

## 0.1.1

### Minor

- Make release builds more reproducible
- Add `platform-rpc` for vendor-specific platform RPCs
- Implement `platform-reboot` command
- Add `--timeout` for platform protocol timeout
- Implement `applet-rpc` command
- Add `--serial` and `WASEFIRE_SERIAL` to select the platform
- Implement `platform-list` command
- Support creating, building, and testing a Rust applet
- Support generating a shell completion file

### Patch

- Publish LICENSE file
- Update dependencies
- Depend on features of dependencies explicitly
- Use common Wasefire lints

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 4 -->
