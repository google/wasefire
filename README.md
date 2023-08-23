# Wasefire

This project aims at making firmware development more accessible and secure by
providing a platform for WebAssembly applets. See the
[book](https://google.github.io/wasefire) for more information.

### :warning: Disclaimer :warning:

This is not an officially supported Google product.

This is a **research** project and should be considered **experimental**. In
particular, the project does not provide any guarantees. However, the project
will try to follow the [`cargo` SemVer
guidelines](https://doc.rust-lang.org/cargo/reference/semver.html) for its
crates. Currently, all crates are considered **unstable**.

## Quick start

### Repository setup

Clone the repository and run the setup script:

```sh
git clone https://github.com/google/wasefire
cd wasefire
./scripts/setup.sh
```

Please refer to <https://google.github.io/wasefire/quick> for details about the
setup script.

### Running an applet

Run the applet `$name` in language `$lang` using the `host` runner (i.e. on your
machine without an actual board):

```sh
# You can use name=hsm or name=ctap for example (lang=rust for both).
cargo xtask applet $lang $name runner host
# This will ask for your password because sudo is needed to setup USB/IP.
```

Here's a quick explanation of the command line:
- `cargo` is the (extendable) Rust package manager
- `xtask` is a custom subcommand implemented in the `xtask` crate
- `applet $rust $name` selects the applet to compile
- `runner host` selects the runner to compile to, here the host machine

The `hsm` applet implements some HSM-like API on USB serial. The `ctap` applet
implements some CTAP-like API on USB serial.

If you have a nRF52840 development board, you can build and flash a release
with:

```sh
cargo xtask --size --release applet $lang $name runner nordic
```

Here's an explanation of the additional arguments:
- `--size` prints size information for the applet and the platform.
- `--release` does not compile debugging facilities (like logging).

### Interacting with the hsm applet

There is a companion tool in `examples/rust/hsm/host` to interact through USB
serial with the applet:

```sh
cd examples/rust/hsm/host
cargo run -- generate-key 0
cargo run -- encrypt 0 --input=plain.txt --output=cipher.bin
cargo run -- decrypt 0 --input=cipher.bin
cargo run -- export-key 0 --output=key.bin
```

### Interacting with the ctap applet

You can directly interact with the applet through USB serial:

```sh
picocom -q /dev/ttyACM1
```

You may use another terminal emulator than `picocom` and you may have a
different character device path than `/dev/ttyACM1`.
