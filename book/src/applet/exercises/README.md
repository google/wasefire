# Exercises

The
[`examples/rust/exercises`](https://github.com/google/wasefire/tree/main/examples/rust/exercises)
contains exercises to implement an applet that behaves like a security key over
UART.

The `part-<n>` directories contain the successive parts towards the final
applet. You will need to modify those applets by fixing the different `TODO`
comments. The exercise description is at the top of the `src/lib.rs` file.

The `part-<n>-sol` directories contain the solution for each part. You don't
need to modify those applets. You can look at them for hints while working
`part-<n>`.

The `client` directory contains a binary to communicate with the applet. You
don't need to modify this binary. You can use it by running `cargo run` in that
directory. In particular you can get help with `cargo run -- help` and run send
specific requests to the applet with `cargo run -- register foo` for example.

The `interface` directory contains a library defining the interface between the
applet and the client. You don't need to modify this library but you need to
read its documentation. You will use it from the applet.

You can run an applet on the host runner with:

```shell
cargo xtask applet rust exercises/part-1 runner host --web
```
