# Examples

There are a few [existing
applets](https://github.com/google/wasefire/tree/main/examples/rust) that
demonstrate simple usage of the prelude and should cover all functionalities in
the prelude. Each example starts with a short documentation in its `src/lib.rs`
file.

Noticeable examples are:
- `hsm` implements some simple HSM-like API using the `crypto`, `store`, and
  `usb::serial` modules of the prelude. It comes with a companion program to
  interact with the applet (see the documentation in the `src/lib.rs` of the
  applet).
- `ctap` implements some simple CTAP-like API using the `button`, `clock`,
  `led`, `scheduling`, `store`, and `usb::serial` modules of the prelude. It
  describes its usage when connecting to the USB serial interface.
- `memory_game` implements some memory game using the `usb::serial` module of
  the prelude. It describes its usage when connecting to the USB serial
  interface.
