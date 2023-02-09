# Timers

In this section, we will walk through the `button_abort` example in Rust. It
uses the first button and the first LED of the board. On a short press, the LED
will start blinking. On a long press, the LED will stop blinking. While the
button is pressed, the LED indicates whether the press is short or long:
- The LED is on while the press is short.
- The LED turns off once the press is long.

This applet will need a shared state to know whether the LED must be blinking or
not. We cannot simply use a boolean because the state will be shared. We cannot
use `Cell<bool>` neither because the state must be in the heap[^heap]. So we use
`Rc<Cell<bool>>` which is a common pattern when using callbacks:

```rust,no_run,noplayground
{{#include timer.rs:blinking}}
```

We can now allocate a timer for the blinking behavior using `clock::Timer::new`.
This function takes the handler that will be called each time the timer fires.
The handler simply toggles the LED if we must be blinking. Note how we must move
a clone of the state to the callback. This is also a common pattern when using
callbacks, because callbacks must be `'static`[^heap]:

```rust,no_run,noplayground
{{#include timer.rs:blink}}
```

The rest of the code is done in an infinite loop:

```rust,no_run,noplayground
{{#include timer.rs:loop}}
```

At each iteration, we start by setting up a `button::Listener` to record whether
the button was pressed and then released. The logic is similar to the callback
setup for the timer. The small difference is that we won't need to call any
function on the listener so we prefix its variable name `_button` with an
underscore. We cannot simply omit the variable name because we don't want to
drop it until the end of the loop iteration, otherwise we would stop listening
to button events.

```rust,no_run,noplayground
{{#include timer.rs:button}}
```

We then wait until the button is pressed using `scheduling::wait_until()`. This
function takes a condition as argument and only executes callbacks until the
condition is satisfied.

```rust,no_run,noplayground
{{#include timer.rs:pressed}}
```

According to the specification of this example applet, when the button is
pressed we must turn on the LED (and stop blinking if we were blinking) to
signal a possible short press. Note that callbacks can only executed when a
scheduling function is called, so we are essentially in a critical section. As
such, the order in which we do those 3 lines doesn't matter. However, a callback
might be scheduled before we stop the `blink` timer. It will execute next time
we call a scheduling function. This is ok because when that will happen, the
`blinking` state will be `false` and the `blink` handler will do nothing.

```rust,no_run,noplayground
{{#include timer.rs:led_on}}
```

To detect a long press, we need to start a timer. There is nothing new here
except the `Timer::start()` function which takes the timer mode (one-shot or
periodic) and its duration.

```rust,no_run,noplayground
{{#include timer.rs:timeout}}
```

We now wait for the first event between the button being released and the
timeout firing. This is simply done by using a condition which is a disjunction
of the events of interest. This is a common pattern when implementing behavior
with a timeout.

```rust,no_run,noplayground
{{#include timer.rs:released}}
```

To signal that the timeout was reached or the button was released, we turn off
the LED.

```rust,no_run,noplayground
{{#include timer.rs:led_off}}
```

Finally, if the press was short (i.e. the button was released before the
timeout), we start blinking. This demonstrates the use of periodic timers.

```rust,no_run,noplayground
{{#include timer.rs:short}}
```

There are a few things to note:
- The code is implicit in Rust, but the button handler and the timer handler
  within the loop iteration are dropped before the next iteration. This means
  that their callbacks are unregistered. This could be done explicitly by
  calling `core::mem::drop()` on their variable if needed.
- It is not needed to start and stop the `blink` timer within the loop as long
  as it is started before entering the loop. This is just an optimization to
  avoid calling the handler when we know that the `blinking` shared state is
  `false`, because the handler would do nothing in that case.

The final code looks like this:

```rust,no_run
{{#include timer.rs:all}}
```
## Testing

As for the LEDs and buttons examples, to test the applet on the `host` runner,
you'll need to use:

```shell
cargo xtask applet rust button_abort runner host --log=info
```

However, in addition to `button` which does a press and release sequence, you
can use `press` and `release` to independently press and release the button. In
particular, `button` may be used to start blinking and `press` may be used to
stop blinking. There's no need to explicitly release because the applet supports
missing callbacks for robustness.

[^heap]: If the state were on the stack and a callback were pointing to that
    state, it would become a safety requirement to unregister the callback
    before popping the state from the stack. However, it is safe to leak a
    callback with `core::mem::forget()` and thus not drop it. So we enforce
    callbacks to be `'static` and thus not depend on references to the stack.
