# Storage

For now only a key-value store is supported for persistent storage. Eventually,
additional facilities may be added: a cyclic logging journal, a file-system,
raw flash access, etc.

In this section, we will illustrate the key-value store usage by walking through
the `store` example which provides direct store access through USB serial.

We first define a helper to write a line to the USB serial.

```rust,no_run,noplayground
{{#include store.rs:writeln}}
```

Because values may be at most 1023 bytes, there is a system to store large
entries as multiple fragments of at most 1023 bytes each using multiple keys. To
support those large entries, we define an abstract notion of keys. An abstract
key is either exactly one key, or a contiguous range of keys.

```rust,no_run,noplayground
{{#include store.rs:key}}
```

We then define helpers to dispatch to the regular or fragmented version based on
the abstract key. The `insert`, `find`, and `remove` functions will be explained
later.

```rust,no_run,noplayground
{{#include store.rs:helpers}}
```

The first thing we do when the applet starts is print a short help describing
how to use the applet.

```rust,no_run,noplayground
{{#include store.rs:usage}}
```

We can then start the infinite loop processing exactly one command per
iteration.

```rust,no_run,noplayground
{{#include store.rs:loop}}
```

We start a loop iteration by reading a command from the user. Only space,
lower-case alphabetic characters, digits, and backspace are supported. We exit
as soon as the user hits Enter.

```rust,no_run,noplayground
{{#include store.rs:read}}
```

We then parse the command (described later). If the command is invalid, we print
a message and continue to the next loop iteration.

```rust,no_run,noplayground
{{#include store.rs:parse}}
```

And we finally process the command (described later). If processing failed, we
print a message with the error. Regardless of error, this is the end of the loop
(and thus the main function) and we continue to the next iteration.

```rust,no_run,noplayground
{{#include store.rs:process}}
```

To ease parsing and processing, we define a straightforward type for commands.

```rust,no_run,noplayground
{{#include store.rs:command}}
```

The parsing function is also straightforward.

```rust,no_run,noplayground
{{#include store.rs:impl_parse}}
```

The process function is a `Command` method which may return a store error.

```rust,no_run,noplayground
{{#include store.rs:process_signature}}
```

For insert commands, we simply forward to the `store::insert()` function (resp.
`store::fragment::insert()` for fragmented entries) which maps a key (resp. a
range of keys) to a value. If the key (resp. range of keys) was already mapped,
it is overwritten. A key must be a number smaller than 4096. A range of keys
must be non-empty. A value must be a slice of at most 1023 bytes (resp. 1023
bytes times the number of fragments).

```rust,no_run,noplayground
{{#include store.rs:process_insert}}
```

Remove commands are also straightforward. We use `store::remove()` which maps a
key to nothing. It's not an error if the key wasn't mapped before.

```rust,no_run,noplayground
{{#include store.rs:process_remove}}
```

Finally, find commands are implemented using `store::find()` which takes a key
and return the mapped value if any.

```rust,no_run,noplayground
{{#include store.rs:process_find}}
```

We print a message if no value was found.

```rust,no_run,noplayground
{{#include store.rs:process_none}}
```

Otherwise, we try to convert the byte slice to a string slice. This should
succeed for values that were inserted by this applet since we only accept
alphanumeric characters. In that case, we simply print the value.

```rust,no_run,noplayground
{{#include store.rs:process_ok}}
```

However, because the store is persistent and keys are not yet partitioned by
applets, we could read the values written by a previous applet for that key. And
those values don't need to be valid UTF-8. In those cases, we print the value as
a byte slice.

```rust,no_run,noplayground
{{#include store.rs:process_err}}
```

The final code looks like this:

```rust,no_run
{{#include store.rs:all}}
```
