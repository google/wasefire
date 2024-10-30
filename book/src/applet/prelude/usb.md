# USB

For now only USB serial is supported. Eventually, the idea would be for applets to describe the USB
interfaces they need in some init function. The scheduler would then create the USB device based on
those information. And only then start the applets with capabilities to the interfaces they asked
for.

In this section, we will illustrate USB serial usage by walking through the `memory_game` example.
The game is essentially an infinite loop of memory questions. The player has 3 seconds to memorize a
random base32 string (the length is the current level in the game and thus represents the
difficulty). The player then has 7 seconds to type it back. On success they go to the next level,
otherwise to the previous level.

The applet has only 2 states across loop iterations:

- The level of the game (and thus the length of the string to remember) starting at 3.
- The next prompt to show to the player while they get ready for the next question.

```rust,no_run,noplayground
{{#include usb.rs:state}}
```

Everything else is in the infinite loop:

```rust,no_run,noplayground
{{#include usb.rs:loop}}
```

First thing we do is print the prompt and wait for the player to press Enter. We use ANSI escape
codes to overwrite whatever was there before. As an invariant throughout the game, we always use a
single line of the terminal. This is particularly important to overwrite the question since the
player has to guess it. We write to the USB serial using `serial::write_all()`. This function is
generic over objects implementing `serial::Serial`, in this case `usb::serial::UsbSerial`.

```rust,no_run,noplayground
{{#include usb.rs:prompt}}
```

We then wait until the player presses Enter. We can read a single byte from the USB serial using
`serial::read_byte()`. The terminal sends `0x0d` when Enter is pressed.

```rust,no_run,noplayground
{{#include usb.rs:ready}}
```

To generate the next question, we use `rng::fill_bytes()` which fills a buffer with random bytes. We
provide a buffer with the length of the current level. For the string to be printable we truncate
the entropy of each byte from 8 to 5 bits and convert it to a `base32` symbol.

```rust,no_run,noplayground
{{#include usb.rs:generate}}
```

We can now show the question to the player. We do so using a `process` helper function that we will
also use for the answer. We instantiate this function such that the player has 3 seconds to memorize
the question and may hit Enter at any time to start answering.

```rust,no_run,noplayground
{{#include usb.rs:question}}
```

After 3 seconds have elapsed or if the player hit Enter, we read the answer from the player. We give
them 7 seconds to type the answer. We also convert lower-case letters to upper-case for convenience
(it's easier to read upper-case but easier to type lower-case). We also support backspace which the
terminal sends as `0x7f`. And same as for the question, we let the player exit early with Enter to
avoid waiting until the timeout.

```rust,no_run,noplayground
{{#include usb.rs:answer}}
```

Once we have the answer, we check if it matches the question. If it does, we promote the player to
the next level. If it doesn't, we demote the player to the previous level. However, if there are no
previous level because the player is at level 1, then we let them retry the level to show our
support. We use ANSI escape codes to highlight the result.

```rust,no_run,noplayground
{{#include usb.rs:update}}
```

Now that we're done with the main loop, let's look at the `process` helper. It takes 4 arguments:

- `max_secs: usize`: the maximum display time in seconds.
- `prompt: &str`: the message shown at the beginning of the line.
- `data: &mut String`: the data shown after the prompt, which may be updated (see below).
- `update: impl Fn(&mut String, u8) -> bool`: the closure called on each input byte possibly
  updating the data and returning whether processing should end immediately without waiting for the
  maximum display time.

```rust,no_run,noplayground
{{#include usb.rs:process}}
```

The helper counts the number of elapsed seconds in shared variable `secs` and updates it using a
periodic timer every second.

```rust,no_run,noplayground
{{#include usb.rs:process_setup}}
```

The helper loops as long as the update function didn't say to stop (tracked by the `done` variable)
and there is still time available.

```rust,no_run,noplayground
{{#include usb.rs:process_loop}}
```

We update the line in the terminal with the prompt, time left, and current data. We use ANSI escape
codes to highlight the data and help readability.

```rust,no_run,noplayground
{{#include usb.rs:process_write}}
```

To be able to update the time left in the terminal we must read from the USB serial asynchronously
using `serial::Reader`. We create a reader by providing a mutable buffer to which the reader will
write the received bytes.

```rust,no_run,noplayground
{{#include usb.rs:process_reader}}
```

We then sleep until a callback is executed using `scheduling::wait_for_callback()`. This callback
may either be the timer firing the next second or the reader getting input from the USB serial.

```rust,no_run,noplayground
{{#include usb.rs:process_wait}}
```

We call `Reader::result()` to know how many bytes were read from USB serial and written to the
buffer (or if an error occurred). We then simply iterate over the received bytes and update the data
and early exit status according to the provided closure. Same as with timers, when a reader is
dropped, its callback is canceled.

```rust,no_run,noplayground
{{#include usb.rs:process_read}}
```

The final code looks like this:

```rust,no_run
{{#include usb.rs:all}}
```

## Testing

The host platform doesn't enable USB serial by default. Pass `--usb-serial` to enable it:

```shell
wasefire host --usb-serial
```

Once the applet is running, you can connect to the USB serial with the following command:

```shell
picocom -q /dev/ttyACM0
```
