# [Introduction](https://google.github.io/wasefire)

This book is a walk through the _Wasefire_ project.

## Vision

> Provide a **simple-to-use** and **secure-by-default** ecosystem for firmware development.

### Simple-to-use

The ecosystem aspires to be accessible to projects and developers regardless of their familiarity
with developing secure firmware:

- No embedded expertise required: Regular software engineers can develop firmware.
- No security expertise required: The ecosystem provides secure-by-default options.
- No enforced programming language: Developers may use the language of their choice[^pl-choice].
- No enforced development environment or build system: Developers may use their usual setup.

[^pl-choice]: As long as it compiles to WebAssembly (or the target architecture for native applets).

### Secure-by-default

Security is the responsibility of the ecosystem, not the developer. The following security
mechanisms are (or are planned to be) in place:

- Sandboxing of the firmware functions (called applets) from each other and from the firmware
  runtime (called platform).
- Secure implementation within the platform boundaries (e.g. side-channel resistance, fault
  injection protection, etc).
- Security reviews for the supported boards (e.g. side-channel attacks, fault injection, etc).
- User documentation (and warnings) when the security must rely on user behavior, for example when a
  configuration is insecure.

Even though the default is to be secure, the ecosystem will provide options to opt-out from some of
the security features for performance or footprint reasons. In other words, the developer is able to
make the trade-off between security and performance, defaulting to security without explicit action.

### Symbiosis

Being both simple-to-use and secure-by-default actually goes hand in hand. It cannot be expected for
humans to never do mistake, even if they have embedded and security expertise.

- By being simple to use, developers will prefer using the ecosystem solution rather than
  implementing their own, thus using a secure-by-default solution rather than a possibly insecure
  implementation.
- By not being concerned with security and embedded details, developers can be more productive and
  focus on the actual firmware behavior, following the well-lit path for all security and embedded
  questions.

## Non-goals

The project is not trying to build a self-service applet store. In particular, users are developers.
A self-service applet store may come at a later phase or may be done independently by another
project. The project will provide anything needed for such applet store to be secure and easy to
use.

## Disclaimer

The project is still work in progress and many components might change in the future. However, the
project follows the usual [Rust semantic
versioning](https://doc.rust-lang.org/cargo/reference/semver.html) to avoid unexpected breakages.
