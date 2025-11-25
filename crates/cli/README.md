# Wasefire CLI

## Installation

You can install the CLI from crates.io with cargo:

```sh
cargo install wasefire-cli
```

The first time the CLI will execute, it will first update itself with the [latest version from
GitHub](https://github.com/google/wasefire/releases/latest), as if `wasefire self-update` was used.
The reason is that for `wasefire host` to work, the CLI must embed a host platform, which is not
published and must be built separately.

## Usage

The CLI is self-descriptive:

```sh
wasefire help
```

## Shell completion

You can generate a completion file for your shell:

```sh
wasefire completion --output=path/to/completion/dir/wasefire
```

If you don't know the completion directory for your shell, you may install it at the system level.
For example, for bash:

```sh
sudo wasefire completion bash --output=/etc/bash_completion.d/wasefire
```
