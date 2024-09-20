# Changelog

## 0.2.0-git

### Major

- Change API to be async using tokio

### Minor

- Change the flags of `action::AppletRpc` to use `action::Wait`
- Add `action::Wait` for commands returning an optional response
- Increase the default connection timeout from 1 to 5 seconds
- Add `action::PlatformUpdate` for platform update
- Add `action::PlatformList` to list connected platforms
- Add `action::ConnectionOptions` for commands that need a platform connection
- Change the behavior of `fs::copy_if_changed()` to keep an original source

### Patch

- Update dependencies

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 5 -->
