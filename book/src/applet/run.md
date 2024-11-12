# Run an applet

If you don't have a Wasefire platform, you can start a development platform on your machine
according to the [quick start](../quick/index.html) instructions.

You can install a Rust applet with the following command:

```shell
wasefire rust-applet-install
```

This command does two things:

- It builds the Rust applet, like `wasefire rust-applet-build` would.
- It installs the built applet, like `wasefire applet-install wasefire/applet.wasm` would.
