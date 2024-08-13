# Changelog

## 0.2.0-git

### Major

- Change all actions requiring a connection to use async
- Change the return type of `ConnectionOptions::connect()`
- Change `action::PlatformList::run()` and remove `action::PlatformInfo`

### Minor

- Add `action::PlatformList` to list connected platforms
- Add `action::ConnectionOptions` for commands that need a platform connection
- Change the behavior of `fs::copy_if_changed()` to keep an original source

### Patch

- Update dependencies

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 2 -->
