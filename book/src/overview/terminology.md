# Terminology

The project will refer to the following components:

- A [Device](#device) is a final product with possibly multiple functions.
- A [User](#user) is a person, team, or organization owning the Device.
- A [Board](#board) is the hardware of the Device.
- An [Applet](#applet) is part of the software of the Device implementing a function.
- The [Board API](#board-api) is the hardware abstraction layer.
- The [Applet API](#applet-api) is the platform abstraction layer.
- A [Prelude](#prelude) is a library providing language-specific access to the Applet API.
- A [Runner](#runner) is a binary implementing the Board API and calling the Scheduler.
- The [Scheduler](#scheduler) is the software implementing all the platform logic. It sits between
  the Board API and the Applet API.
- A [Platform](#platform) is the binary made of a Runner and the Scheduler.

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

Users are responsible for all those steps. The project will however support them in those tasks for
both development and security aspects. For example (non-exhaustive list):

- The platform provides a serial (ideally unique) for the device.
- The platform supports secure updates.
- The platform provides applet management (with versioning).
- The platform supports some existing hardware (may add support for more based
  on demand).

## Board

The project provides support for some boards through Runners. Additional boards may be supported
depending on User needs.

Boards may be subject to some restrictions:

- Only ARM Cortex-M and RISC-V architectures are considered for now.
- Minimum flash and RAM requirements (to be defined).

## Applet

The functionality (or business logic) of a Device is implemented as a set of concurrent applets. The
project provides example applets in the different supported languages for the User to use as
starting point. Eventually, an applet is compiled into WebAssembly which is the only language that
the Scheduler supports[^lang-support].

[^lang-support]: There is currently support for native applets as a work-around performance issues.
    Such applet is compiled to the target architecture and linked with the platform.

## Board API

The Board API is the interface between a Board and the Scheduler and is implemented by a Runner. It
is specified by the `wasefire-board-api` crate and provides cargo features to select the relevant
part of the API. This selection may be motivated by different reasons:

- The Board doesn't support some features in hardware and a software implementation is not feasible
  or desired.
- The User knows that some features are not going to be used by any applets that will run on the
  Device. And the space saved by not implementing them is needed for the Device to properly
  function.

## Applet API

The Applet API is the interface between an Applet and the Scheduler. It is specified by the
`wasefire-applet-api-desc` crate which can generate code for different languages (e.g. the
`wasefire-applet-api` crate for Rust). It also provides cargo features to select the relevant part
of the API.

The Board API and Applet API relate by the following points:

- Together, they provide portability of Applets across different Boards.
- They are similar interfaces but with major differences.
- The Board API is low-level and trusted by the Scheduler, while the Applet API may be filtered by
  the Scheduler based on permissions and capabilities.

## Prelude

The Applet API is defined at the WebAssembly level. Applet developers may use it directly, but
depending on the language this usage may not be convenient. For example, in Rust, using the Applet
API requires the `unsafe` keyword and thus some good understanding of what's going on. For such
languages, a Prelude is provided to give a simpler and more natural interface to the Applet API. For
example, in Rust, the `wasefire` crate provides a safe API with high-level Rust-specific types like
`&[u8]` (instead of the `u32` pointer and `u32` length that the Applet API expects).

## Runner

The project provides Runners for supported Boards. However, Board support doesn't need to be
provided by the project. The User (for example the team developing the Board) may develop its own
Runner for its own Board. Simplifying the development of Runners (by maximizing the code shared with
other Runners) is part of the project vision. The project simplifies development of both Runners and
Applets, not just Applets.

## Scheduler

Together with the Board API and the Applet API, the Scheduler is the core contribution of the
project and where most security properties are implemented. This is completely provided by the
project and Users cannot alter it. However, they can configure it when configuring a Platform.

## Platform

This is the firmware that runs on the Device. It doesn't provide any business logic, but provides
core functionalities through the platform protocol (non-exhaustive list):

- Secure updates.
- Applet management.
- Debugging facilities.
