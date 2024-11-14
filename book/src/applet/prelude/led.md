# LEDs

In this section, we will walk through the `blink` example in Rust. It will blink in order each LED
of the board every second in an infinite loop (going back to the first LED after the last LED).

The number of LEDs available on the board is advertised by the `led::count()` function. We want to
make sure there is at least one LED available:

```rust,no_run,noplayground
{{#include led.rs:count}}
```

We start the infinite loop cycling through all LEDs in order:

```rust,no_run,noplayground
{{#include led.rs:loop}}
```

Within the infinite loop (notice the indentation), we first turn on the current LED using the
`led::set()` function:

```rust,no_run,noplayground
{{#include led.rs:set}}
```

We now wait for half a second because we want to blink each LED for one second which means half a
second on and half a second off:

```rust,no_run,noplayground
{{#include led.rs:sleep}}
```

Finally, we repeat the same process but turning the LED off before looping to the next LED:

```rust,no_run,noplayground
{{#include led.rs:repeat}}
```

The final code looks like this:

```rust,no_run
{{#include led.rs:all}}
```

## Testing

The `host` runner currently has 1 LED and, either prints its state to `stdout` (the default) or
blinks the LED in the Web UI (with `--interface=web`). Eventually, the number of LEDs will be
configurable.

To build and install the applet on a connected platform:

```shell
wasefire rust-applet-install
```

The output in a host platform without Web UI (i.e. `wasefire host`) should look like this:

```text
Applet running.
Led is on
Led is off
Led is on
Led is off
Led is on
Led is off
Led is on
Led is off
Led is on
Led is off
Led is on
Led is off
Led is on
Led is off
Led is on
The applet was killed.
```

Where the applet was uninstalled (and thus killed) with `wasefire applet-uninstall`.
