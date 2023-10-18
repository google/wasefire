# UART

Using the UART is similar to using the USB serial, because the
`uart::Uart(uart_id)` object implements `serial::Serial`, where `uart_id` is the
UART id. The `uart::count()` function returns how many UARTs are available on
the device. UART ids must be smaller than this count.

It is usually a good idea to write generic code over any serial without assuming
a particular implementation. This can be done by using a `serial` variable
implementing `serial::Serial`. This variable may be instantiated differently
based on a compilation feature:

```rust,no_run,noplayground
#[cfg(feature = "serial_uart")]
let serial = uart::Uart(0);
#[cfg(feature = "serial_usb")]
let serial = usb::serial::UsbSerial;
// ...
serial::write_all(&serial, b"hello").unwrap();
```

When using the host runner, you can connect to the UART with:

```shell
socat -,cfmakeraw UNIX-CONNECT:target/wasefire/uart0
```
