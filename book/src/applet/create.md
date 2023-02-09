# Create a new applet

This step will eventually be a simple `cargo wasefire new <applet-name>` command
out of tree. But for now we will build the applet within the project repository
as an example applet. We'll use `tutorial` as the applet name throughout this
tutorial.

You have 2 options to create and populate the applet directory. We'll go over
both for pedagogical reasons.

## Copy the `hello` applet

The first step is to copy the `hello` directory to the `tutorial` directory:

```shell
cp -r examples/rust/hello examples/rust/tutorial
```

The second step is to update the applet name in the `Cargo.toml`:

```shell
sed -i 's/hello/tutorial/' examples/rust/tutorial/Cargo.toml
```

## Create the applet from scratch

Create the `tutorial` directory:

```shell
mkdir examples/rust/tutorial
```

Create the `Cargo.toml` file in the created directory with the following
content:

```toml
[package]
name = "tutorial"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
prelude = { path = "../../../crates/prelude" }
```

The `crate-type` entry is needed to compile to Wasm. The `prelude` dependency
provides a high-level interface to the Applet API. Eventually, this dependency
will released on `crates.io` and you will be able to develop out-of-tree.

Then create the `src/lib.rs` file in the created directory with the following
content:

```rust,no_run
{{#include create.rs:all}}
```

Let's go over and explain each line.

```rust,no_run,noplayground
{{#include create.rs:nostd}}
```

The applet must be no-std to prevent accidental dependency on `std` which is not
supported for the `wasm32-unknown-unknown` target. Note that using WASI with the
`wasm32-wasi` target is not a workaround because the Applet API doesn't (yet?)
provide WASI.

```rust,no_run,noplayground
{{#include create.rs:alloc}}
```

If the applet uses the global allocator, then it must re-enable allocation
support (which was disabled by the no-std line before). The prelude provides the
global allocator but this line is still needed because it must be in the binary.

Note that because of those 2 lines, you need to use `core` or `alloc` instead of
`std`.

```rust,no_run,noplayground
{{#include create.rs:prelude}}
```

This line is for convenience and imports everything the prelude defines. You can
instead import the items you need manually.

```rust,no_run,noplayground
{{#include create.rs:nomangle}}
```

Rust mangles the name of all functions. But we want the `main` function to have
a predictible name such that the scheduler can find and call it when loading the
applet. This line disables mangling.

```rust,no_run,noplayground
{{#include create.rs:externc}}
```

This line ensures the `main` function has a correct foreign function interface.
