# Wasefire Web Client

This is a fairly minimal web app for interacting with the runner that's built
with [Yew](https://yew.rs) and [Trunk](https://trunkrs.dev/).

## Running

```bash
trunk serve
```

Rebuilds the app whenever a change is detected and runs a local server
to host it.

There's also the `trunk watch` command which does the same thing but without
hosting it.

## Release

```bash
trunk build --release
```

This builds the app in release mode similar to `cargo build --release`.
You can also pass the `--release` flag to `trunk serve` if you need to get
every last drop of performance.

Unless overwritten, the output will be located in the `dist` directory.
