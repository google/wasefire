# LEDs

In this section, we will walk through the `blink` example in Rust. It will blink
in order each LED of the board every second in an infinite loop (going back to
the first LED after the last LED).

The number of LEDs available on the board is advertised by the `led::count()`
function. We want to make sure there is at least one LED available:

```rust,no_run,noplayground
{{#include led.rs:count}}
```

Next, we initialize the variables describing the action of the next loop
iteration. The very first action we want to take is turn on the first LED
(indexing starts at 0).

```rust,no_run,noplayground
{{#include led.rs:init}}
```

We can now start the infinite loop:

```rust,no_run,noplayground
{{#include led.rs:loop}}
```

Within the infinite loop (notice the indentation), we first take the action
described by the variables which is to set the status of a LED using the
`led::set()` function:

```rust,no_run,noplayground
{{#include led.rs:action}}
```

We now need to update the variables to describe the next action. We always flip
the status because we want a blinking behavior (`led::Status` implements `Not`
so we can use the `!` notation). And if the next status is to turn on a LED,
then we want to turn on the next LED, taking care of wrapping around to the
first LED after the last LED:

```rust,no_run,noplayground
{{#include led.rs:update}}
```

Finally, before looping back to the next iteration, we sleep for half a second
because we want to blink each LED for one second which means half a second on
and half a second off:

```rust,no_run,noplayground
{{#include led.rs:sleep}}
```

The final code looks like this:

```rust,no_run
{{#include led.rs:all}}
```

## Testing

The `host` runner currently has 1 LED and prints its state on `stdout` as an
info-level log. Eventually, the number of LEDs will be configurable and how they
are represented will be improved (for example through some graphical interface).

To test the applet on the `host` runner, you'll thus need to use:

```shell
cargo xtask applet rust blink runner host --log=info
```

The `--log=info` flag specifies that we want info-level (or more severe)
logging. By default, only errors are printed.
