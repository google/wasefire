# Changelog

## 0.2.0-git

### Major

- Change API to be async using tokio

### Minor

- Add `action::PlatformLock` for locking a platform protocol
- Expose `action::Transfer` for transfers from host to device
- Add `action::Wait::set_period()` and `impl Default for action::Wait`
- Add `action::AppletExitStatus` to get the applet exit status
- Add `action::Applet{Install,Uninstall}` for simple applet management
- Add `action::PlatformApiVersion` to get a platform API version
- Change the flags of `action::AppletRpc` to use `action::Wait`
- Add `action::Wait` for commands returning an optional response
- Change the default connection timeout from 1 seconds to infinite (0 seconds)
- Add `action::PlatformUpdate` for platform update
- Add `action::PlatformList` to list connected platforms
- Add `action::ConnectionOptions` for commands that need a platform connection
- Change the behavior of `fs::copy_if_changed()` to keep an original source

### Patch

- Update dependencies

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 5 -->
