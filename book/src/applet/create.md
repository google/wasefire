# Create a new applet

You can create a Rust applet called `tutorial` with the following command:

```sh
wasefire rust-applet-new tutorial
```

This creates a `tutorial` directory with an example Rust applet. Let's look at `Cargo.toml`:

- The `wasefire` dependency provides a Rust interface for the applet API.
- The `wasefire-stub` optional dependency provides support for testing.
- The `test` feature enables testing support.

And now let's look at `src/lib.rs`:

- The `#![no_std]` attribute is needed for building WASM modules without WASI. As a consequence, you
  cannot use `std`. Instead, you need to use `core` and `alloc`.
- The `wasefire::applet!();` macro does a few things:
   - It makes sure the `main()` function is executed when the applet starts.
   - It imports the `alloc` crate.
   - It imports all the items of the `wasefire` crate.
- In the `tests` module, the `use wasefire_stub as _;` makes sure the `wasefire-stub` crate is
  linked since it provides symbol definitions for the applet API.

Since most commands assume they are running from the root directory of the Rust applet (unless
`--crate-dir` is provided), you can change the working directory to the crate:

```sh
cd tutorial
```
