# Security key example applet using OpenSK

This applet is an example of how to implement a security key on Wasefire. It depends on the
[OpenSK](https://github.com/google/OpenSK) library to implement the authenticator side of CTAP.

## Setup

To follow any of the instructions below, you need to clone the repository and execute the setup
script (which assumes a Debian-like system like Ubuntu to install USB-, SSL-, and build-related
packages):

```sh
git clone https://github.com/google/wasefire.git
cd wasefire
./scripts/setup.sh
```

The setup script is idempotent, so you can always rerun it, either because you don't remember you've
run it or because you just pulled a newer version of the `main` branch with `git pull`.

## Features

The applet provides a few customization features (all disabled by default):

- `ctap1` enables support for CTAP 1 (the applet always implements CTAP 2)
- `ed25519` enables support for Ed25519 (the applet always implements ECDSA P-256)
- `fingerprint` enables support for fingerprints

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

```sh
export APPLET_FEATURES=--features=ctap1,ed25519
```

Similarly, the `PLATFORM_FEATURES` environment variable controls the platform features to enable
depending on the enabled applet features. So it must be either unset, or set using a `--features=`
prefix. Each platform will describe whether it supports each applet feature, and which platform
features to enable if it does.

Note that you may need to run `./scripts/setup.sh` if any command fails.

## Targets

Only native and pulley applets are supported, due to the size and complexity of the applet. It is
not possible to use a wasm applet at this time. This limitation may be lifted in the future.

The commands below are given for native applets. For pulley applets, the command must be run twice,
replacing `--native` with `--pulley` and:

- For the first run, removing everything between `applet` (included) and `runner` (excluded).
- For the second run, replacing everything after `runner` (included) with `install`.

For example, if the native command is:

```sh
cargo xtask --native applet rust opensk $APPLET_FEATURES runner foo $PLATFORM_FEATURES flash
```

Then the 2 pulley commands (respectively for the platform and the applet) would be:

```sh
cargo xtask --pulley runner foo $PLATFORM_FEATURES flash
cargo xtask --pulley applet rust opensk $APPLET_FEATURES install
```

### Host

#### Feature: ctap1

The applet feature `ctap1` is supported and doesn't need any platform feature.

#### Feature: ed25519

The applet feature `ed25519` is supported and doesn't need any platform feature.

#### Feature: fingerprint

The applet feature `fingerprint` is not supported.

#### Target: native

```sh
cargo xtask --native applet rust opensk $APPLET_FEATURES \
  runner host flash --usb-ctap --interface=web
```

#### Target: pulley

Start the platform in its own terminal:

```sh
cargo xtask --pulley runner host flash --usb-ctap --interface=web
```

Install the applet from another terminal:

```sh
cargo xtask --pulley applet rust opensk $APPLET_FEATURES install
```

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

```sh
cargo xtask --release --native \
  applet rust opensk --opt-level=z $APPLET_FEATURES \
  runner nordic --opt-level=z --features=usb-ctap $PLATFORM_FEATURES \
    --features=software-crypto-aes256-cbc,software-crypto-hmac-sha256 \
    --features=software-crypto-p256-ecdh,software-crypto-p256-ecdsa \
  flash
```

#### Board: Dongle

Make sure the dongle is in DFU mode by plugging it while holding the reset button.

```sh
cargo xtask --release --native \
  applet rust opensk --opt-level=z $APPLET_FEATURES \
  runner nordic --board=dongle --opt-level=z --features=usb-ctap $PLATFORM_FEATURES \
    --features=software-crypto-aes256-cbc,software-crypto-hmac-sha256 \
    --features=software-crypto-p256-ecdsa,software-crypto-p256-ecdh \
  flash
```

This command will eventually pause and instruct you to enter DFU mode again (by pressing the reset
button) then hit Enter to continue.

#### Board: Makerdiary

Make sure the [makerdiary](https://makerdiary.com/products/nrf52840-mdk-usb-dongle-w-case) is in DFU
mode by plugging it while holding the button. The LED should be green. Also make sure the USB mass
storage device class is mounted. It should appear as UF2BOOT.

```sh
cargo xtask --release --native \
  applet rust opensk --opt-level=z --features=led-1 $APPLET_FEATURES \
  runner nordic --board=makerdiary --opt-level=z --features=usb-ctap $PLATFORM_FEATURES \
    --features=software-crypto-aes256-cbc,software-crypto-hmac-sha256 \
    --features=software-crypto-p256-ecdsa,software-crypto-p256-ecdh \
  flash
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

```sh
cargo xtask --release --native \
  applet rust opensk --opt-level=z $APPLET_FEATURES \
  runner opentitan --opt-level=z --features=usb-ctap $PLATFORM_FEATURES \
  flash
```
