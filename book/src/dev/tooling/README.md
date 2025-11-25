# Tooling

Most of the tooling is handled by `cargo xtask` which is an alias defined in `.cargo/config.toml` to
run the binary under `crates/xtask`. This command must run from the root of the repository. All
command-line arguments are documented and accessible from `cargo xtask help`. The most important
ones are:

- `cargo xtask runner` to compile (and possibly flash or update) a platform
- `cargo xtask applet` to compile (and possibly install) an applet

Some tools are still implemented as shell scripts under the `scripts` directory. The most important
ones are:

- `./scripts/ci.sh` to run the full (software) continuous integration locally
- `./scripts/hwci.sh` to run the hardware continuous integration on a specific device
- `./scripts/sync.sh` to synchronize all generated content (part of the CI)
- `./scripts/wrapper.sh` to run a dependency after installing it if needed

[Ideally](https://github.com/google/wasefire/issues/208), all shell scripts would migrate to `cargo
xtask`.
