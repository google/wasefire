# Create a new applet

This step will [eventually](https://github.com/google/wasefire/issues/38) be a
simple `wasefire new <applet-name>` command out of tree. But for now we will
build the applet within the project repository as an example applet. We'll use
`tutorial` as the applet name throughout this tutorial.

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
crate-type = ["cdylib"] # needed for building wasm

[dependencies]
wasefire = "*" # use the latest version
```

The `crate-type` entry is needed to compile to Wasm. The `wasefire` dependency
provides a high-level interface to the Applet API.

Then create the `src/lib.rs` file in the created directory with the following
content:

```rust,no_run
#![no_std] // needed for building wasm (without wasi)
wasefire::applet!(); // imports the prelude and defines main as entry point

fn main() {
    println!("hello world");
}
```

Note that because you need to use `core` or `alloc` instead of `std`.
