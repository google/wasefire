# Quick start

## Repository setup

Clone the repository and run the setup script:

```sh
git clone https://github.com/google/wasefire
cd wasefire
./scripts/setup.sh
```

The setup script is best effort and is only meant for Unix systems so far. On
Debian-based Linux systems, the script will directly invoke installation
commands, possibly asking for sudo password. On other systems, the script will
return an error and print a message each time a binary or library must be
installed. Once the package is installed, the script must be run again. The
script will eventually print nothing and return a success code indicating that
no manual step is required and everything seems in order.

## Run an applet

If you don't have a Nordic board, you can run applets on your desktop:

```sh
cargo xtask applet assemblyscript hello runner host
```

If you have an nRF52840 dev-kit, you can also run applets on the board:

```sh
cargo xtask applet rust blink runner nordic
```

The general format is `cargo xtask applet LANGUAGE NAME runner BOARD`. Example
applets are listed in the `examples` directory by _language_ then _name_.
_Boards_ are listed under the `crates` directory and are prefixed by `runner-`.

Feel free to stop and play around by directly editing the examples. Or continue
reading for a deeper tutorial on how to write applets in Rust.
