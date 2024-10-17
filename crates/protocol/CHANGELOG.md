# Changelog

## 0.2.0-git

### Major

- Remove `applet::Response` and inline its definition in `AppletResponse::Response`

### Minor

- Add `PlatformLock` to lock a platform protocol
- Add `AppletExitStatus` and `applet::ExitStatus` to get an applet exit status
- Add `Applet{Install,Uninstall}` for applet management
- Add `ConnectionExt::call_ref()` to share a request between calls
- Add a `Service::NAME` constant with `host` feature
- Add `PlatformUpdate{Metadata,Transfer}` calls and `transfer` module for platform updates
- Add a `Connection` abstraction with `host` feature

### Patch

- Update dependencies

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 4 -->
