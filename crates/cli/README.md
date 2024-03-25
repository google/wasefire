# Wasefire CLI

## Installation

You can install the CLI from crates.io with cargo:

```
cargo install wasefire-cli
```

There will eventually be binary releases on Github too.

## Usage

The CLI is self-descriptive:

```
wasefire help
```

## Shell completion

You can generate a completion file for your shell:

```
wasefire completion --output=path/to/completion/dir/wasefire
```

If you don't know the completion directory for your shell, you may install it at the system level.
For example, for bash:

```
sudo wasefire completion bash --output=/etc/bash_completion.d/wasefire
```
