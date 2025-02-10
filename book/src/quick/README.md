# Quick start

Please open an [issue](https://github.com/google/wasefire/issues/new) if a step doesn't work.

## Download the Wasefire CLI

### Local machine

Download the [latest release](https://github.com/google/wasefire/releases/latest) of the Wasefire
CLI for your platform. The code sample below assumes that your platform is
`x86_64-unknown-linux-gnu` and that your `PATH` contains `~/.local/bin`. You may need to adapt the
commands for a different platform or environment.

```shell
tar xf wasefire-x86_64-unknown-linux-gnu.tar.gz
rm wasefire-x86_64-unknown-linux-gnu.tar.gz
mkdir -p ~/.local/bin
mv wasefire-x86_64-unknown-linux-gnu ~/.local/bin/wasefire
```

You can test that the CLI is correctly installed by running `wasefire help`.

You can also add shell completion with `wasefire completion`. You need to place the generated script
where you shell will interpret it, which depends on your shell and configuration. If you use bash
and have root access, you can copy it to `/etc/bash_completion.d/wasefire` or
`/usr/share/bash-completion/completions/wasefire`.

### GitHub Codespace

- Open <https://codespaces.new/google/wasefire?quickstart=1>
- Click the green `Create new codespace` (or `Resume this codespace`) button
- Wait a couple minutes for the codespace to be created

## Start a host platform

You can start a host platform for development with the following command (it will run until
interrupted or killed, so you will need a dedicated terminal):

```shell
wasefire host --interface=web
```

You may omit `--interface=web` if you don't need to interact with buttons and LEDs.

This will create a `wasefire/host` directory in the current directory to store the state of the host
platform.

This will also ask for sudo permissions when using the USB platform protocol, which is the default
(except in GitHub Codespace where `export WASEFIRE_PROTOCOL=unix` is added to the `.bashrc` file).
If you don't want to use sudo, you can use the `unix` or `tcp` platform protocol. You'll have to
pass `--protocol=unix` or `--protocol=tcp` to most `wasefire` commands or set the
`WASEFIRE_PROTOCOL` environment variable in your shells.

## Hello world in Rust

You can create a new Rust applet with:

```shell
wasefire rust-applet-new hello
cd hello
```

This will create a directory called `hello` with an example "hello world" applet.

You can build this applet (from the `hello` directory) with:

```shell
wasefire rust-applet-build
```

You can also run the unit tests with:

```shell
wasefire rust-applet-test
```

And you can install it (building it if needed) on a connected platform (for example the host
platform started earlier) with:

```shell
wasefire rust-applet-install
```

Regardless of the programming language, if you already built an applet (by default under
`wasefire/applet.wasm`), you can install it with:

```shell
wasefire applet-install wasefire/applet.wasm
```

And you can uninstall an applet (regardless of programming language) with:

```shell
wasefire applet-uninstall
```

Wasefire supports only one applet at a time for now. Once multiple applets can be installed
simultaneously, the `applet-uninstall` command will take an applet ID as argument.
