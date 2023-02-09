Applet-side of the applet API.

This `api` crate provides Rust bindings for the Wasm-side of the applet API.
Users should not directly use those bindings but instead use the `prelude` crate
that provides higher-level and more convenient access.

The bindings in this crate are automatically generated from a proc-macro based
on a procedural description of the applet API defined in the `api-desc` crate.
As such, they follow a precise convention:
- The module hierarchy in the applet API is mapped to the module hierarchy in
  this crate.
- Type definitions (like enums) are mapped to their definition. If an enum is
  used as an error, it provides a `Error::to_result(isize) -> Result<usize,
  Error>` method to convert signed integer results into Rust `Result`.
- Functions are mapped to their definition but also to a module of the same
  name. This module defines 2 structs: `Params` for the parameters of the
  function and `Results` for the results of the function. If a struct is empty,
  then it is not used.
- All parameters and results are 32-bits integers mapping to the `i32` Wasm
  type, but their Rust type reflect their role. For example:
  - `usize` for unsigned integers
  - `isize` for signed integers (which may represent some `Result<u31, u31>`
    where the error type may be converted to some enum)
  - `*const u8` and `*mut u8` for opaque/polymorphic pointers
  - `*const T` and `*mut T` for transparent pointers where `T` is recursively
    defined
  - `extern "C" fn(data: *mut u8, ...)` for closures over some opaque data and
    possibly additional parameters
