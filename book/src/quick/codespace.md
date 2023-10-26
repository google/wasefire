# Github codespace tips

## Editing an applet

For rust-analyzer to work properly, you need to open the applet in its own
workspace, for example:

```shell
code examples/rust/exercises/part-1
```

However, this will also modify the working directory of the terminal. To be able
to run `cargo xtask`, you will need to go back to the root of the git
repository:

```shell
cd /workspaces/wasefire
```

You can then open the `src/lib.rs` file and benefit from documentation (hovering
a name), auto-completion, diagnostics, and other rust-analyzer features.
