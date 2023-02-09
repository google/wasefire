# Setup

The list of required dependencies might not be complete. So if anything in the
tutorial doesn't go according to plan, please open an
[issue](https://github.com/google/wasefire/issues/new) describing the problem.

If you don't have Rust installed, follow [the official
instructions](https://rustup.rs/) to install `rustup`.

Make sure to use a nightly compiler by default:

```shell
rustup default nightly
```

Make sure to have the `wasm32-unknown-unknown` target:

```shell
rustup target add wasm32-unknown-unknown
```

If you get linking errors due to `libusb`, you might need to uninstall
`libusb-dev` and `libusb-0.1-4` to only keep `libusb-1.0-0-dev`, see [this
issue](https://github.com/a1ien/rusb/issues/117) for more information.

## Clone the project repository

This step will eventually not be needed. Because the project is not yet released
on `crates.io`, the prelude and other dependencies must be available somewhere
on the filesystem.

```shell
git clone https://github.com/google/wasefire
cd wasefire
git submodule update --init
```
