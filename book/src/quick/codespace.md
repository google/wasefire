# GitHub Codespace tips

## Editing an applet

For rust-analyzer to work properly, you need to open the applet in its own workspace, for example:

```shell
code examples/rust/exercises/part-1
```

You can then open the `src/lib.rs` file and benefit from documentation (hovering a name),
auto-completion, diagnostics, and other rust-analyzer features.

## Troubleshooting

If you get an error about `lsmod` not found, you must run `exec bash` to make sure the
`WASEFIRE_PROTOCOL` environment variable is set. The `.bashrc` file is modified as part of the
Codespace setup and it may happen that the initial terminal was started before the setup finished.
