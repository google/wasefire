# Run an applet

We currently use `cargo xtask` as an alias to the local `xtask` crate to build,
flash, and run platforms and applets. Eventually, this will be a `wasefire`
command and will work out-of-tree. You can use `cargo xtask help` to discover
the tool.

## On host

We can run an applet (here the `tutorial` applet) on the `host` runner with the
following command:

```shell
cargo xtask applet rust tutorial runner host
```

Type your password when asked. The `host` runner needs `sudo` to set USB/IP up,
which is needed for applets that use USB. This is disabled in Github Codespace.
After a bunch of compilation steps, you should see something that ends like:

```plaintext
     Running `.../wasefire/target/release/runner-host`
Executing: sudo modprobe vhci-hcd
[sudo] password for user:
Executing: sudo usbip attach -r localhost -b 1-1
Board initialized. Starting scheduler.
00000.000: hello world
```

The first line is output by `cargo`. The last 2 lines are output by the host
runner. The last one was triggered by the applet. Debugging output is prefixed
with a timestamp.

The host runner (like all runners) doesn't stop, even if all applets have
completed. Instead, it goes to sleep. This is because all known use-cases are
reactor-like (they react to external input). Besides, if the platform has applet
management enabled, then the platform is ready to execute applet management
commands. However, if there is a use-case that needs to shutdown, then the API
or scheduler will be extended to provide this functionality.

Use Ctrl-C to terminate the runner. For hardware boards, you can just remove the
power or let it run. The device is in sleep state (although if USB is enabled,
then it wakes up every millisecond to keep the connection active).

## On board

We currently only support the nRF52840-dk board from Nordic. If you have such a
dev board, you can run an applet (here the `tutorial` applet) on the `nordic`
runner with the following command:

```shell
cargo xtask applet rust tutorial runner nordic
```

After a bunch of compilation steps, you should see something that ends like:

```plaintext
".../wrapper.sh" "probe-rs" "run" "--chip=nRF52840_xxAA" "target/.../runner-nordic"
     Erasing sectors [00:00:05] [################################]
 Programming pages   [00:00:04] [################################]
0.090527: hello world
```

The first line is from `cargo xtask`. The rest is from `probe-rs run`. The last
line is triggered by the applet. Debugging output is prefixed by a timestamp in
seconds.
