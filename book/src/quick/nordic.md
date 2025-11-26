# Nordic bootstrap

Download the [latest release](https://github.com/google/wasefire/releases/latest) of the Wasefire
platform for your board and follow the instructions of the appropriate section below. You can verify
that the Wasefire platform is working by listing the connected Wasefire platforms:

```sh
wasefire platform-list
```

Note that this platform only provides basic functionality. Depending on your final use-case, you may
need to update the platform with `wasefire platform-update`.

## nRF52840 DK

These instructions are for the [nRF52840
DK](https://www.nordicsemi.com/Products/Development-hardware/nRF52840-DK).

Install the `nrfutil` program following the [official instructions][nrfutil]. Then install the
`device` command and make sure your device is the only one listed:

```sh
nrfutil install device
nrfutil device list
```

Erase the device, program the firmware, and reset the device:

```sh
nrfutil device erase
nrfutil device program --firmware=platform-nordic-devkit.hex
nrfutil device reset
```

## nRF52840 Dongle

These instructions are for the [nRF52840
Dongle](https://www.nordicsemi.com/Products/Development-hardware/nRF52840-Dongle).

Install the `nrfutil` program following the [official instructions][nrfutil]. Then install the
`device` and `nrf5sdk-tools` commands, and generate the package for each step (the signature warning
is expected):

```sh
nrfutil install device
nrfutil install nrf5sdk-tools
nrfutil nrf5sdk-tools pkg generate --hw-version=52 --sd-req=0 \
  --application-version=0 --application=platform-nordic-dongle-1.hex \
  platform-nordic-dongle-1.zip
nrfutil nrf5sdk-tools pkg generate --hw-version=52 --sd-req=0 \
  --application-version=0 --application=platform-nordic-dongle-2.hex \
  platform-nordic-dongle-2.zip
```

Make sure the device is in DFU mode by pressing the reset button (the small horizontal button
labeled `RESET` next to the big vertical button). The second LED should pulse red in a breathing
pattern.

Make sure your device is the only one listed:

```sh
nrfutil device list
```

Flash the first package:

```sh
nrfutil device program --traits=nordicDfu --firmware=platform-nordic-dongle-1.zip
```

Return the device to DFU mode by pressing the reset button, then flash the second package:

```sh
nrfutil device program --traits=nordicDfu --firmware=platform-nordic-dongle-2.zip
```

## nRF52840 MDK USB Dongle

These instructions are for the [nRF52840 MDK USB
Dongle](https://makerdiary.com/products/nrf52840-mdk-usb-dongle-w-case).

Download the `uf2conv.py` script from the [official
repository](https://raw.githubusercontent.com/makerdiary/nrf52840-mdk-usb-dongle/refs/heads/master/tools/uf2conv.py).

Make sure the device is in DFU mode by plugging it while holding the button. The LED should be
green. Also make sure the USB mass storage device class is mounted. It should appear as `UF2BOOT`.
Flash the device:

```
python3 uf2conv.py --family=0xADA52840 platform-nordic-makerdiary.hex
```

Replug the device to reset it.

[nrfutil]: https://docs.nordicsemi.com/bundle/nrfutil/page/guides/installing.html
