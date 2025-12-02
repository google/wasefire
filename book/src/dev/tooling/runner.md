# Compile a platform

The words _platform_ and _runner_ are often used interchangeably. The word _runner_ refers to the
binary itself, while _platform_ refers to the functionality provided by the binary. The runners are
found under the `crates` directory and have their name prefixed with `runner-`, like `runner-host`
and `runner-nordic`.

There are many options to compile (and possibly flash or update) a platform, all described by `cargo
xtask help runner` and its descendants like `cargo xtask help runner flash`. Here are a few examples
for illustration purposes:

```sh
# Compile a Nordic platform with debugging enabled (panics and errors).
cargo xtask runner nordic

# Same as above but also flash the firmware to the unique connected board.
cargo xtask runner nordic flash

# Same as above but flash to the specified board (in case there are many).
cargo xtask runner nordic flash --probe=$VID:$PID:$serial

# Compile a Nordic platform with more debugging enabled.
cargo xtask runner nordic --log=info

# Compile a Nordic platform with full debugging for a specific crate.
cargo xtask runner nordic --log=info,runner_nordic=trace

# Compile a Nordic platform with additional features.
cargo xtask runner nordic --features=gpio,uart,usb-serial

# Compile a Nordic platform without debugging (the --log argument is ignored).
cargo xtask --release runner nordic

# Compile and flash a Nordic platform for the dongle (instead of the dev-kit).
cargo xtask --release runner nordic --board=dongle flash
```

To verify that a platform was correctly flashed, you can list all connected Wasefire devices:

```sh
cargo wasefire platform-list
```
