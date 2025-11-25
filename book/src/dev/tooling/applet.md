# Compile an applet

The repository contains example applets under the `examples` directory. They are also used for
testing by `scripts/ci.sh` and `scripts/hwci.sh`. The applets are categorized by language and most
applets are under `examples/rust`.

The options to compile (and possibly install) an applet are described by `cargo xtask help applet`
and `cargo xtask help applet install`. Here are the most common examples:

```sh
# Compile the timer_test Rust applet.
cargo xtask applet rust timer_test

# Same as above but also installs the applet to the unique connected board.
cargo xtask applet rust timer_test install

# Same as above but also waits until the applet terminates.
cargo xtask applet rust timer_test install wait
```
