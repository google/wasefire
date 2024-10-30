# Exercises

The
[`examples/rust/exercises`](https://github.com/google/wasefire/tree/main/examples/rust/exercises) of
the Wasefire repository contains exercises to implement an applet that behaves like a security key
over UART.

The `part-<n>` directories contain the successive parts towards the final applet. You will need to
modify those applets by fixing the different `TODO` comments. The exercise description is at the top
of the `src/lib.rs` file.

The `part-<n>-sol` directories contain the solution for each part. You don't need to modify those
applets. You can look at them for hints while working `part-<n>`.

The `client` directory contains a binary to communicate with the applet. You don't need to modify
this binary. You can build it with `cargo run --release` from the `client` directory, then execute
it from the host platform directory with `$WASEFIRE_REPO/target/release/client`. You can also copy
the executable to the host platform directory and use `./client`. You need to run it from the host
platform directory because it assumes the UART socket to be at `wasefire/host/uart0` (which is the
default for the host platform).

The `interface` directory contains a library defining the interface between the applet and the
client. You don't need to modify this library but you need to read its documentation. You will use
it from the applet.
