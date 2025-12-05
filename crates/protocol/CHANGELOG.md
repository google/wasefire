# Changelog

## 0.4.0-git

### Major

- Use `Cow<'a, [u8]>` instead of `&'a [u8]` to support `serde`

### Minor

- Add `Service::response()` with `serde` feature
- Implement `Serialize` and `Deserialize` for `Api` with `serde` feature

### Patch

- Update dependencies

## 0.3.0

### Major

- Add `Api::AppletReboot` to reboot an applet
- Deprecate `transfer::Request` to `transfer::_Request0`
- Deprecate `Api::AppletUninstall` to `Api::_AppletUninstall0`
- Deprecate `Api::AppletInstall` to `Api::_AppletInstall0`
- Add `Api::PlatformClearStore` to clear the store
- Add `Any` bound to `Connection`
- Deprecate `platform::Info` to `platform::_Info0`
- Deprecate `Api::PlatformInfo` to `Api::_PlatformInfo0`
- Deprecate `Api::PlatformUpdateMetadata` to `Api::_PlatformUpdateMetadata`
- Rename `Api::PlatformUpdateTransfer` to `Api::PlatformUpdate`

### Minor

- Add `transfer::Response` for more granular erase
- Fix lints
- Add `platform::Info` to support A/B platform update
- Use Rust edition 2024

### Patch

- Update dependencies

## 0.2.0

### Major

- Remove `applet::Response` and inline its definition in `AppletResponse::Response`

### Minor

- Add `serde` feature
- Add `PlatformLock` to lock a platform protocol
- Add `AppletExitStatus` and `applet::ExitStatus` to get an applet exit status
- Add `Applet{Install,Uninstall}` for applet management
- Add `ConnectionExt::call_ref()` to share a request between calls
- Add a `Service::NAME` constant with `host` feature
- Add `PlatformUpdate{Metadata,Transfer}` calls and `transfer` module for platform updates
- Add a `Connection` abstraction with `host` feature

### Patch

- Fix rust and clippy lints
- Update dependencies

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 3 -->
