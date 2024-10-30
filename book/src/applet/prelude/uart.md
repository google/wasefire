# UART

Using the UART is similar to using the USB serial, because the `uart::Uart::new(uart_id)` object
implements `serial::Serial`, where `uart_id` is the UART index. The `uart::count()` function returns
how many UARTs are available on the device. UART indices must be smaller than this count.

It is usually a good idea to write code that is generic over the serial implementation. This can be
done by using a `serial` variable implementing `serial::Serial`. This variable may be instantiated
differently based on a compilation feature:

```rust,no_run,noplayground
#[cfg(feature = "serial_uart")]
let serial = uart::Uart::new(0).unwrap();
#[cfg(feature = "serial_usb")]
let serial = usb::serial::UsbSerial;

serial::write_all(&serial, b"hello").unwrap();
```

Host platforms don't have a real UART. Instead they create a UNIX socket. You can connect to such a
UART by connecting to the UNIX socket. In a terminal dedicated for the connection (you will need to
close the terminal to close the connection) and from the directory where the host platform is
running, you can run:

```shell
socat -,cfmakeraw UNIX-CONNECT:wasefire/host/uart0
```
