# Acknowledgments

This project would not be where it is now if it couldn't build upon the
following projects[^exhaustiveness]:

| Project            | Author                    | Usage                       |
|--------------------|---------------------------|-----------------------------|
| [defmt]            | [Knurling]                | Logging for embedded        |
| [probe-run]        | [Knurling]                | (now replaced by probe-rs)  |
| [probe-rs]         | [probe.rs]                | Flashing and debugging      |
| [cortex-m]         | [Rust Embedded]           | Cortex-M support            |
| [riscv]            | [Rust Embedded]           | RISC-V support              |
| [embedded-hal]     | [Rust Embedded]           | Hardware abstraction        |
| [embedded-alloc]   | [Rust Embedded]           | Heap allocation             |
| [critical-section] | [Rust Embedded]           | Mutex support               |
| [portable-atomic]  | [taiki-e]                 | Atomic support              |
| [usb-device]       | [Rust Embedded Community] | Generic USB interface       |
| [usbd-serial]      | [Rust Embedded Community] | USB serial implementation   |
| [usbd-hid]         | [twitchyliquid64]         | USB HID implementation      |
| [usbip-device]     | [Sawchord]                | USB/IP implementation       |
| [nrf-hal]          | [nRF Rust]                | nRF52840 support            |
| [nrf-usbd]         | [nRF Rust]                | nRF52840 USB implementation |
| [wasmtime]         | [Bytecode Alliance]       | Pulley applets              |
| [bytemuck]         | [Lokathor]                | Safe low-level casts        |
| [tokio]            | [tokio-rs]                | Host platform concurrency   |
| [hashes]           | [Rust Crypto]             | SHA-256 and 384             |
| [MACs]             | [Rust Crypto]             | HMAC-SHA-256 and 384        |
| [AEADs]            | [Rust Crypto]             | AES-128-CCM and 256-GCM     |
| [elliptic-curves]  | [Rust Crypto]             | NIST P-256 and 384          |
| [rust-analyzer]    | [Rust]                    | Rust development            |
| [wasm3]            | [Wasm3 Labs]              | (not used anymore)          |

We would like to thank all authors and contributors of those projects (as well
as those we use but we forgot to mention) and more generally the community
around Rust for embedded development. We are also grateful to those who answered
our issues [[1], [2], [3], [4], [5], [6], [7], [8], [9]] and reviewed our pull
requests [[10], [11], [12], [13]].

[10]: https://github.com/probe-rs/probe-rs/pull/1919
[11]: https://github.com/nrf-rs/nrf-usbd/pull/17
[12]: https://github.com/Sawchord/usbip-device/pull/5
[13]: https://github.com/rust-embedded-community/usb-device/pull/115
[1]: https://github.com/probe-rs/probe-rs/issues/1865
[2]: https://github.com/rust-embedded/critical-section/issues/42
[3]: https://github.com/probe-rs/probe-rs/issues/1863
[4]: https://github.com/probe-rs/probe-rs/issues/1816
[5]: https://github.com/knurling-rs/probe-run/issues/421
[6]: https://github.com/RustCrypto/traits/issues/1311
[7]: https://github.com/RustCrypto/KDFs/issues/80
[8]: https://github.com/RustCrypto/traits/issues/1307
[9]: https://github.com/knurling-rs/defmt/issues/738
[AEADs]: https://github.com/RustCrypto/AEADs
[Bytecode Alliance]: https://github.com/bytecodealliance
[Knurling]: https://github.com/knurling-rs
[Lokathor]: https://github.com/Lokathor
[MACs]: https://github.com/RustCrypto/MACs
[Rust Crypto]: https://github.com/RustCrypto
[Rust Embedded Community]: https://github.com/rust-embedded-community
[Rust Embedded]: https://github.com/rust-embedded
[Rust]: https://github.com/rust-lang
[Sawchord]: https://github.com/Sawchord
[Wasm3 Labs]: https://github.com/wasm3
[bytemuck]: https://github.com/Lokathor/bytemuck
[cortex-m]: https://github.com/rust-embedded/cortex-m
[critical-section]: https://github.com/rust-embedded/critical-section
[defmt]: https://github.com/knurling-rs/defmt
[elliptic-curves]: https://github.com/RustCrypto/elliptic-curves
[embedded-alloc]: https://github.com/rust-embedded/embedded-alloc
[embedded-hal]: https://github.com/rust-embedded/embedded-hal
[hashes]: https://github.com/RustCrypto/hashes
[nRF Rust]: https://github.com/nrf-rs
[nrf-hal]: https://github.com/nrf-rs/nrf-hal
[nrf-usbd]: https://github.com/nrf-rs/nrf-usbd
[portable-atomic]: https://github.com/taiki-e/portable-atomic
[probe-rs]: https://github.com/probe-rs/probe-rs
[probe-run]: https://github.com/knurling-rs/probe-run
[probe.rs]: https://github.com/probe-rs
[riscv]: https://github.com/rust-embedded/riscv
[rust-analyzer]: https://github.com/rust-lang/rust-analyzer
[taiki-e]: https://github.com/taiki-e
[tokio-rs]: https://github.com/tokio-rs
[tokio]: https://github.com/tokio-rs/tokio
[twitchyliquid64]: https://github.com/twitchyliquid64
[usb-device]: https://github.com/rust-embedded-community/usb-device
[usbd-hid]: https://github.com/twitchyliquid64/usbd-hid
[usbd-serial]: https://github.com/rust-embedded-community/usbd-serial
[usbip-device]: https://github.com/Sawchord/usbip-device
[wasm3]: https://github.com/wasm3/wasm3
[wasmtime]: https://github.com/bytecodealliance/wasmtime

[^exhaustiveness]: We tried to focus on the projects that were critical to get
    where we are now. Some projects are thus deliberately left out (and some
    projects that we don't use anymore are still listed). But it is also
    possible that we forgot to list some, in which case we apologize and would
    be happy to fix our oversight if notified.
