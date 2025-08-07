# Security key example applet using OpenSK

This applet is an example of how to implement a security key on Wasefire. It depends on the
[OpenSK](https://github.com/google/OpenSK) library to implement the authenticator side of CTAP.

## Features

The applet provides a few customization features (all disabled by default):
- `ctap1` enables support for CTAP 1 (the applet always implements CTAP 2)
- `ed25519` enables support for Ed25519 (the applet always implements ECDSA P-256)
- `fingerprint` enables support for fingerprints

## Limitations

The applet can only be compiled natively with a platform. It cannot be compiled as a Web-Assembly
applet at this time. This limitation will be lifted in the future.

## Platforms

The applet needs the platform to implement the following features of the board API:
- `api-button`
- `api-clock`
- `api-crypto-aes256-cbc`
- `api-crypto-ed25519` if the applet `ed25519` feature is enabled
- `api-crypto-hmac-sha256`
- `api-crypto-p256-ecdh`
- `api-crypto-p256-ecdsa`
- `api-crypto-sha256`
- `api-fingerprint-matcher` if the applet `fingerprint` feature is enabled
- `api-led`
- `api-rng`
- `api-storage`
- `api-timer`
- `api-usb-ctap`

In the following sections, we provide instructions to flash an OpenSK applet for each platform
provided by this repository. Those instructions assume that the environment variable
`APPLET_FEATURES` contains the applet features to enable. It must be unset if no features must be
enabled. If set, it should start with `--features=` and continue with a non-empty comma-separated
list of the applet features to enable. For example:

```rust
export APPLET_FEATURES=--features=ctap1,ed25519
```

Similarly, the `PLATFORM_FEATURES` environment variable controls the platform features to enable
depending on the enabled applet features. So it must be either unset, or set using a `--features=`
prefix. Each platform will describe whether it supports each applet feature, and which platform
features to enable if it does.

Note that you may need to run `./scripts/setup.sh` if any command fails.

### Host

It is currently not possible to run OpenSK on the host platform, because it only supports
Web-Assembly applets. This limitation will either be lifted when the OpenSK applet can compile to
Web-Assembly or when the host platform supports running native applets.

### nRF52840

#### Feature: ctap1

The applet feature `ctap1` is supported and doesn't need any platform feature.

#### Feature: ed25519

The applet feature `ed25519` is not supported.

#### Feature: fingerprint

The applet feature `fingerprint` is supported for the development kit if an [FPC
2534](https://www.fingerprints.com/solutions/access/fpc-allkey-development-kit) is connected to the
board. In that case, the `fpc2534` platform feature must be enabled.

An FPC 2532 should theoretically also be supported (but has not been tested) using the same platform
feature.

#### Board: Development kit

```shell
cargo xtask --release --native \
  applet rust opensk --opt-level=z $APPLET_FEATURES \
  runner nordic --opt-level=z --features=usb-ctap $PLATFORM_FEATURES \
    --features=software-crypto-aes256-cbc,software-crypto-hmac-sha256 \
    --features=software-crypto-p256-ecdh,software-crypto-p256-ecdsa \
  flash --reset-flash
```

#### Board: Dongle

Make sure the dongle is in DFU mode by plugging it while holding the reset button.

```shell
cargo xtask --release --native \
  applet rust opensk --opt-level=z $APPLET_FEATURES \
  runner nordic --board=dongle --opt-level=z --features=usb-ctap $PLATFORM_FEATURES \
    --features=software-crypto-aes256-cbc,software-crypto-hmac-sha256 \
    --features=software-crypto-p256-ecdsa,software-crypto-p256-ecdh \
  flash
```

If the board was used before, it may be necessary to clear the storage and reboot:

```shell
cargo wasefire platform-clear-store
cargo wasefire platform-reboot
```

If you want a Wasefire platform that supports platform update, you'll need to pass the
`--dongle-update-support` flag:

```shell
cargo xtask --release --native \
  applet rust opensk --opt-level=z $APPLET_FEATURES \
  runner nordic --board=dongle --opt-level=z --features=usb-ctap $PLATFORM_FEATURES \
    --features=software-crypto-aes256-cbc,software-crypto-hmac-sha256 \
    --features=software-crypto-p256-ecdsa,software-crypto-p256-ecdh \
  flash --dongle-update-support
```

This requires pushing the reset button one additional time during the process (and hitting the Enter
key on the keyboard). Instructions will be printed to the standard output.

#### Board: Makerdiary

Make sure the [makerdiary](https://makerdiary.com/products/nrf52840-mdk-usb-dongle-w-case) is in DFU
mode by plugging it while holding the button. The LED should be green. Also make sure the USB mass
storage device class is mounted. It should appear as UF2BOOT.

```shell
cargo xtask --release --native \
  applet rust opensk --opt-level=z --features=led-1 $APPLET_FEATURES \
  runner nordic --board=makerdiary --opt-level=z --features=usb-ctap $PLATFORM_FEATURES \
    --features=software-crypto-aes256-cbc,software-crypto-hmac-sha256 \
    --features=software-crypto-p256-ecdsa,software-crypto-p256-ecdh \
  flash
```

If the board was used before, it may be necessary to clear the storage and reboot:

```shell
cargo wasefire platform-clear-store
cargo wasefire platform-reboot
```

### OpenTitan

#### Feature: ctap1

The applet feature `ctap1` is supported and doesn't need any platform feature.

#### Feature: ed25519

The applet feature `ed25519` is supported and needs the `software-ed25519` platform feature. Once
the OpenTitan crypto library implements Ed25519, it will be possible to enable the `ed25519`
platform feature instead.

#### Feature: fingerprint

The applet feature `fingerprint` is not supported.

#### Board: Teacup A2

A LED (active high) needs to be connected to R10. A capacitive touch needs to be connected to R13.

```shell
cargo xtask --release --native \
  applet rust opensk --opt-level=z $APPLET_FEATURES \
  runner opentitan --opt-level=z --features=usb-ctap $PLATFORM_FEATURES \
  flash
```

If the board was used before, it may be necessary to clear the storage and reboot:

```shell
cargo wasefire platform-clear-store
cargo wasefire platform-reboot
```
