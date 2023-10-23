# Quick start

Please open an [issue](https://github.com/google/wasefire/issues/new) if a step
doesn't work.

## Repository setup

### Local machine

Clone the repository and run the setup script[^1]:

```shell
git clone https://github.com/google/wasefire
cd wasefire
./scripts/setup.sh
```

### Github codespace

- Open <https://codespaces.new/google/wasefire?quickstart=1>
- Click the green `Create new codespace` (or `Resume this codespace`) button
- Wait 2 minutes for the codespace to be created (or a 10 seconds to be resumed)

## Run an applet

Depending on your hardware, you can run the hello applet with:
- `cargo xtask applet rust hello runner nordic` to run on an nRF52840 dev-kit
- `cargo xtask applet rust hello runner host` to run on your desktop. 
   Add `--features=web` to start a web UI for your board.

The general format is `cargo xtask applet LANGUAGE NAME runner BOARD`. Example
applets are listed in the `examples` directory by _language_ then _name_.
_Boards_ are listed under the `crates` directory and are prefixed by `runner-`.
You can find more details using `cargo xtask help`.

Feel free to stop and play around by directly editing the examples. Or continue
reading for a deeper tutorial on how to write applets in Rust.

[^1]: The setup script is best effort and is only meant for Unix systems so far.
    On Debian-based Linux systems, the script will directly invoke installation
    commands, possibly asking for sudo password. On other systems, the script
    will return an error and print a message each time a binary or library must
    be installed. Once the package is installed, the script must be run again.
    The script will eventually print nothing and return a success code
    indicating that no manual step is required and everything seems in order.
