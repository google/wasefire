# RPC

In this section, we will walk through the `protocol` example in Rust. It converts the case of
alphabetic ASCII characters from its request to its response and switches the letters `I` and `O`
(similarly for lower-case the letters `i` and `o`).

The platform protocol (used to install applets, update the platform, reboot the platform, etc) also
provides a way to call into an applet by sending a request and reading the response. The applet can
define a handler taking a `Vec<u8>` request as argument and returning a `Vec<u8>` response (which
can reuse the backing storage of the request).

In this example, the handler simply converts all characters of the request, prints how many times it
was called, and returns the updated request as its response[^mut].

```rust,no_run,noplayground
{{#include rpc.rs:handler}}
```

A listener can be created and leaked as usual.

```rust,no_run,noplayground
{{#include rpc.rs:listener}}
```

The conversion function is straightforward:

```rust,no_run,noplayground
{{#include rpc.rs:convert}}
```

The final code looks like this:

```rust,no_run
{{#include rpc.rs:all}}
```

## Testing

You can use the `wasefire applet-rpc` command to send an RPC to an applet. By default it reads from
standard input and write to standard output.

```shell
% echo HELLO ping | wasefire applet-rpc
helli PONG
```

[^mut]: Note that we don't need interior mutability here, compared to buttons for example. This is
    because the RPC handler is not re-entrant. The platform won't send a new request if the previous
    request was not responded.
