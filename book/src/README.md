# [Introduction](https://google.github.io/wasefire)

This book is a walk through the _Wasefire_ project.

## Vision

> Make it **easy** and **cheap** to build **secure** firmware.

The 3 aspects of the vision are realized through the 3 following pillars.

### Developer experience

The project tries to integrate with the pre-existing developer environments
instead of creating a new one. The developer should be able to write firmware
business logic the same way they write their non-firmware business logic. This
is similar to the goal of [knurling-rs](https://knurling.ferrous-systems.com)
but for more languages than just Rust.

### Separation of concerns

The project does not require a team to possess multiple expertise because this
is costly. Instead, the project splits the firmware development into
inter-operable components such that each component requires a single expertise.
Development can thus be distributed over multiple teams having their own
expertise (e.g. hardware design, embedded programming, security, and business
logic). The project provides stable and pre-existing APIs for those teams and
components to inter-operate.

### Secure by design

The project is responsible for the security of the final firmware. This project
provides the following security mechanisms:
- Sandboxing of the applets between each other and with the platform.
- Secure implementation within the platform boundaries (e.g. side-channel
  resistance, fault injection protection, etc).
- Security reviews for the supported boards (e.g. side-channel attacks, fault
  injection, etc).
- User documentation when the security must rely on user behavior, for example
  when a configuration is insecure.

## Non-goals

The project is not trying to build a self-service applet store. In particular,
users are developers. A self-service applet store may come at a later phase or
may be done independently by another project. The project will provide anything
needed for such applet store to be secure and easy to use.

## Disclaimer

The project is still work in progress and many components might change in the
future. However, the project follows the usual [Rust semantic
versioning](https://doc.rust-lang.org/cargo/reference/semver.html) to avoid
unexpected breakages.
