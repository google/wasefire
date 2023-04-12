# Frequently asked questions

This section tries to answer common questions. If your question is not listed
here, please open an [issue](https://github.com/google/wasefire/issues/new).

### Are applets trusted to be correct?

No. The platform does not trust applets and applets do not trust each other.
However, if an applet has a valid signature, then the platform trusts the
permissions required by the applet.

### Are applets executed as native code?

No. Applets are installed as WebAssembly byte-code. This is required since the
static guarantees provided by WebAssembly apply to the byte-code and the
platform checks those guarantees. For execution, applets are interpreted: either
directly from the byte-code, or for performance purposes from an optimized
representation in flash or RAM which may be computed ahead-of-time or on-demand.

### Why is performance not an issue?

The main bets are:
- Computing intensive code (like cryptography) is done in hardware or native
  code (in the platform).
- Applets are supposed to only do business logic which is assumed to not be
  computing intensive.
- The platform targets users who can't write embedded code, so the main concern
  is not performance but making firmware development accessible.

### How does this fit on micro-controllers?

The interpreter currently fits in 22kB when optimized for size and 66kB when
optimized for speed. The interpreter is also designed to have minimal RAM
overhead. However, most optimizations (for both performance and overhead) are
not yet implemented, which may increase the binary size.

### Why implement a new interpreter?

The following runtimes have been quickly rejected:
- `wasmtime` and `wasmer` don't support no-std
- `wasmi` consumes too much RAM for embedded

`wasm3` has been used during the initial phase of the project but got eventually
replaced with a custom interpreter for the following reasons:
- It doesn't perform validation
  [yet](https://github.com/wasm3/wasm3/issues/344). We probably need proper
  validation.
- It only compiles to RAM (not flash). We want to be able to preprocess a module
  and persist the pre-computation in flash such that it is only done when a
  module is installed and not each time it is instantiated.
- It takes control of the execution flow. All runtimes I'm aware of behave like
  that. To simplify scheduling, we want the interpreter to give back control
  when asked or when a host function is called.
- It is written in C. Although the code has tests and fuzzing, we want
  additional security provided by the language.

The interpreter we implemented is written in Rust, doesn't take control of the
execution thread, doesn't pre-compute anything yet (but will be able to
pre-compute to flash), and performs validation.

### Applet footprint compared to native code?

WebAssembly byte-code is compact so there should be a footprint benefit compared
to native code. However, no benchmarks have been done in that regard.

### Is it possible to share code between applets?

Yes (although not yet implemented). Applets are represented at runtime by a
WebAssembly store which is unique per applet. Applets behavior is defined by a
set of WebAssembly modules which are instantiated to the applet store. Applets
may share those modules. A typical example would be an allocator module.
Multiple applets may use the same allocator byte-code (from the module) to
manage their own linear memory (from the module instance in the applet store).

### What third-party dependencies are used?

The minimum set of third-party dependencies is currently:
- `num_enum` for the interpreter
- `usb-device` and `usbd-serial` for the board API

Additional dependencies are used by:
- the actual board implementation:
    - (e.g. `cortex-m-rt`, `nrf52840-hal`, `panic-abort` for nordic)
    - (e.g. `tokio`, `usbip-device`, `aes`, `rand` for linux)
- compilation (e.g. `proc-macro2`, `quote`)
- debugging (e.g. `defmt`, `defmt-rtt`, `log`, `env_logger`)
- tooling (e.g. `anyhow`, `clap`)

In particular, the project doesn't need any operating system (e.g. TockOS) but
may use one as part of a board implementation.
