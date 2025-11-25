# Storage

For now only a key-value store is supported for persistent storage. Eventually, additional
facilities may be added: a cyclic logging journal, a file-system, raw flash access, etc.

In this section, we will illustrate the key-value store usage by walking through the `store` example
which provides direct store access with a text-based interface through the platform protocol applet
RPC mechanism.

The key-value store has 2 restrictions:

- Keys must be between 0 and 4095
- Values must be at most 1023 bytes

The second restriction can be worked around by storing large entries as multiple fragments of at
most 1023 bytes each using multiple keys. To support those large entries, we define an abstract
notion of keys. An abstract key is either exactly one key, or a contiguous range of keys.

```rust,no_run,noplayground
{{#include store.rs:key}}
```

The store provides 3 operations: `insert`, `find`, and `remove`. Each of those operation comes with
a fragmented variant (working on a range of keys for large values). We define helpers to dispatch to
the regular or fragmented version based on the abstract key.

```rust,no_run,noplayground
{{#include store.rs:helpers}}
```

To ease parsing and processing of RPC requests, we define a straightforward type for commands.

```rust,no_run,noplayground
{{#include store.rs:command}}
```

The parsing function is also straightforward.

```rust,no_run,noplayground
{{#include store.rs:impl_parse}}
```

The process function is more interesting as we'll describe how the store operations behave. When
processing a command, we may either succeed with an output, or fail with an error. We use strings
because we implement a text-based interface.

```rust,no_run,noplayground
{{#include store.rs:process_signature}}
```

For the help command, we simply output the grammar for commands.

```rust,no_run,noplayground
{{#include store.rs:process_help}}
```

For insert commands, we use our helper function for abstract keys and format the output and error
appropriately. When inserting to the store, if the key (or range of keys) is already present, then
its value is overwritten.

```rust,no_run,noplayground
{{#include store.rs:process_insert}}
```

Remove commands are similar. When removing from the store, it is not an error if the key (or range
of keys) is absent.

```rust,no_run,noplayground
{{#include store.rs:process_remove}}
```

Find commands also use the helper function, however they have 2 possible outputs (outside errors):

- If the key is absent from the store, then `None` is returned.
- If the key is present, then `Some` is returned with the bytes of the value. Because we implement a
  text-based interface, we try to convert the byte slice to a string slice for the output.

```rust,no_run,noplayground
{{#include store.rs:process_find}}
```

We can finally write our handler function taking a request as argument and returning a response.

```rust,no_run,noplayground
{{#include store.rs:handler}}
```

The main function simply registers an RPC listener with the handler above.

```rust,no_run,noplayground
{{#include store.rs:main}}
```

The final code looks like this:

```rust,no_run
{{#include store.rs:all}}
```

## Testing

We can use the following command to interact with the applet:

```sh
( echo help; cat ) | wasefire applet-rpc --repl
```

An example interaction could look like this:

```text
Usage: insert <key>[..<key>] <value>
Usage: find <key>[..<key>]
Usage: remove <key>[..<key>]
> insert 0 hello world
Error: Invalid command "insert"
> insert 0 hello-world
Done
> find 0
Found: hello-world
> find 1
Not found
> remove 0
Done
> find 0
Not found
```
