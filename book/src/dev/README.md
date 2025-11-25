# Developer guide

This chapter is primarily targeted at potential contributors. However, it can also be used by
advanced users for functionalities not yet available to normal users.

After cloning the repository and for your convenience, you may run the setup script:

```sh
./scripts/setup.sh
```

The script is idempotent, so it is always safe to run (if there's nothing to do, it won't do
anything). It will install dependencies at user-level (like `rustup`) and system-level (like
`openssl`, `pkg-config`, `usbip`, and `libudev`).

The script only supports Debian-like systems at this time. If it doesn't work for you (or you don't
want to run it), you'll simply have to address failures when they occur (possibly taking the script
as inspiration for resolution) based on the error message.
