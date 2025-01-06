### Applet functions

An applet must expose 3 functions to the platform:

- `init: () -> ()` called exactly once before anything else. It cannot call any
  platform function except `dp` (aka `debug::println()`).
- `main: () -> ()` called exactly once after `init` returns. It can call all
  platform functions.
- `alloc: (size: u32, align: u32) -> (ptr: u32)` may be called at any time after
  `init` returns. It cannot call any platform function. A return value of zero
  indicates an error and will trap the applet. The applet may assume that `size`
  is a non-zero multiple of `align` and `align` is either 1, 2, or 4.

### Platform functions

The platform exposes many functions. Each function comes with its own
documentation, which may omit the following general properties.

#### Parameters and result

At WebAssembly level, all functions use `u32` (more precisely `i32`) as
parameters and result (WebAssembly permits multiple results, but platform
functions never return more than one value). However, platform functions will be
documented with more specific types to better convey the intent.

Languages that compile to WebAssembly may use similar specific types when
binding platform functions. Those types should be compatible with `u32` in
WebAssembly.

#### Applet closures

Some functions (usually called `register`) may register a closure. There is
usually an associated function (usually called `unregister`) that unregisters
the registered closure. The "register" function takes `fn { data: *const void,
... }` (usually called `handler_func`) and a `*const void` (usually called
`handler_data`) as arguments. The "unregister" function (if it exists) doesn't
have any particular parameters.

After the closure is registered (the "register" function is called) and as long
as it stays registered (the "unregister" function is not called), the platform
may call this closure any time `sw` (aka `scheduling::wait_for_callback()`) is
called. Note that if `main` returns and there are registered closures, the
applet behaves as if `sw` is called indefinitely.

An applet closure may call any platform function unless explicitly documented by
the "register" function.

#### Reading and writing memory

Some functions take pointers as argument (usually `*const u8` or `*mut u8`).
They either document the expected size or take an associated length argument.

The platform may read and write (only for `*mut T` pointers) the designated
region of the WebAssembly memory during the function call.

#### Allocating memory

Finally, some functions take nested pointers as argument (usually `*mut *mut
u8`). They either return a `usize` or take a `*mut usize` as argument.

Those functions may call `alloc` (at most once per such nested pointer argument)
with a dynamic size but fixed alignment (usually 1). If they do, they will store
the non-null result in the `*mut *mut u8` argument. They always store the size
in the `*mut usize` argument (if it exists) or return it (otherwise).

Note that `alloc` will not be called with a size of zero. So if the size is zero
after the function returns, this signifies that `alloc` was not called (and the
`*mut *mut u8` not written) and an empty output (unless otherwise documented
like in the case of an absence of output).

The caller is responsible for managing the allocation after the function returns
and the pointer is non-null. To avoid memory leaks, the applet should initialize
the `*mut *mut u8` argument to a null pointer and consider having ownership of
the pointer if it is non-null after the call.
