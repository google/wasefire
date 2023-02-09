# Terminology

The project is split in the following components:
- A _Device_ is a final product.
- A _User_ is a person, team, or organization owning the Device.
- A _Board_ is the hardware of the Device.
- An _Applet_ is part of the software of the Device.
- The _Board API_ is the hardware abstraction layer.
- The _Applet API_ is the platform abstraction layer.
- A _Prelude_ is a library providing language-specific access to the Applet API.
- A _Runner_ is a binary implementing the Board API and running the Scheduler.
- The _Scheduler_ is the software implementing all the platform logic and fits
  between the Board API and the Applet API.
- A _Platform_ is the binary made of a Runner and the Scheduler.

## Device

A Device encompasses the following (non-exhaustive list):
- A hardware on which to run (chip, form factor, external devices, etc).
- How this hardware is configured and initially provisioned.
- A set of applets defining the firmware, and their configuration.
- What is the funtionality expected from the firmware.
- Should the Device be certified.
- Where will the Device be installed.
- Who will/should have access to the Device.

## User

Users may delegate part of the Device design to other teams:
- Developing hardware or selecting an existing hardware.
- Developing one or more applets and/or selecting one or more applets.
- Design of the functionality.
- Lifetime management (provisioning, logging, monitoring, alerting, census,
  etc).

Users are responsible for all those steps. The project will however support them
in those tasks for both development and security aspects. For example
(non-exhaustive list):
- The platform provides unique ids per device.
- The platform supports secure updates.
- The platform provides applet management (with versioning).
- The platform supports some existing hardware (may add support for more based
  on demand).

## Board

The project provides support for some boards through Runners. Additional boards
may be supported depending on User needs.

Boards may be subject to some restrictions:
- Only ARM Cortex-M and RISC-V architectures are considered for now.
- Minimum flash and RAM requirements (to be defined).

## Applet

The functionality (or business logic) of a Device is implemented as a set of
concurrent applets. The project provides example applets in the different
supported languages for the User to use as starting point. Eventually, an applet
is compilet into WebAssembly which is the only language that the Scheduler
supports.

## Board API

The Board API may be partially implemented for 2 reasons:
- The Board doesn't support some features in hardware and a software
  implementation is not feasible or desired.
- The User knows that those features are not going to be used by any applets
  that will run on the Device. And the space saved by not implementing or
  building them is needed for the Device to properly function.

## Applet API

The Applet API is currently the same as (or very close to) the Board API. This
may change in the future, for example if capabilities require it. The Board API
is low-level and doesn't have a notion of capabilities. It fully trusts the
Scheduler. The Applet API on the other hand may need to prove to the Scheduler
that it is allowed to access some dynamically-allocated resource.

The Board API and the Applet API provide portability of Applets across different
Boards.

## Prelude

The Applet API is defined at the WebAssembly level. Applet developers may use it
directly, but depending on the language this usage may not be convenient. For
example, in Rust, using the Applet API requires the `unsafe` keyword and thus
some good understanding of what's going on. For such languages, a Prelude is
provided to give a simpler and more natural interface to the Applet API. For
example, in Rust, the Prelude provides a safe API with high-level Rust-specific
types like `&[u8]` (instead of the `u32` pointer and `u32` length that the
Applet API expects).

## Runner

The project provides Runners for supported Boards. However, Board support
doesn't need to be provided by the project. The User (for example the team
developing the Board) may develop its own Runner for its own Board. Simplifying
the development of Runners (by maximizing the code shared with other Runners) is
part of the project vision. The project simplifies development of both Runners
and Applets, not just Applets.

## Scheduler

Together with the Board API and the Applet API, the Scheduler is the core
contribution of the project and where most security properties are implemented.
This is completely provided by the project and Users cannot alter it. However,
they can configure it when configuring a Platform.

## Platform

This is the firmware that runs on the Device. It doesn't provide any business
logic, but provides some core functionalities like (non-exhaustive list):
- Secure updates.
- Applet management.
- Debugging facilities.
