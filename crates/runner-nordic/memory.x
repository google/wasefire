/* nrf52840-dk

RAM:
- 0x20000000 stack (64KiB)
- 0x20010000 data and heap (192KiB)
- 0x20040000

Flash:
- 0x00000000 bootloader (64KiB)
- 0x00010000 platform A (320KiB + 128KiB)
- 0x00060000 platform B (320KiB + 128KiB)
- 0x000b0000 applet (256KiB - 256KiB)
- 0x000f0000 persistent storage (64KiB)
- 0x00100000

*/

/* Keep those values in sync with the header crate used by the bootloader. */
__bootloader_size = 0x00010000;
__platform_size = 0x00050000 + RUNNER_NATIVE * 0x00020000;
__applet_size = (1 - RUNNER_NATIVE) * 0x00040000;
ASSERT(__bootloader_size + 2 * __platform_size + __applet_size == 0x000f0000, "bad layout");
__header_origin = __bootloader_size + RUNNER_SIDE * __platform_size;
__header_length = 0x00000100;
__flash_origin = __header_origin + __header_length;
__flash_length = __platform_size - __header_length;
__sother = __bootloader_size + (1 - RUNNER_SIDE) * __platform_size;
__eother = __sother + __platform_size;
/* Keep those values in sync with --reset-flash in xtask */
__sapplet = __bootloader_size + 2 * __platform_size;
__eapplet = __sapplet + __applet_size;
__sstore = __eapplet;
__estore = 0x00100000;

__stack_origin = 0x20000000;
__stack_length = 0x00010000;
__heap_origin = __stack_origin + __stack_length;
__heap_length = 0x00040000 - __stack_length;

MEMORY {
  HEADER : ORIGIN = __header_origin, LENGTH = __header_length
  FLASH  : ORIGIN = __flash_origin,  LENGTH = __flash_length
  RAM    : ORIGIN = __heap_origin,   LENGTH = __heap_length
}

SECTIONS {
  .header : {
    /* Keep this section in sync with the bootloader. */
    LONG(RUNNER_VERSION);
    /* The 3 boot attempts of the newest side. It's not unusual for 2 attempts
       to be burned at the same time. Not clear why, maybe probe-rs. */
    LONG(0xffffffff);
    LONG(0xffffffff);
    LONG(0xffffffff);
    /* We fill the unused part of the header to keep it erased. */
    FILL(0xffffffff);
    . = ORIGIN(FLASH) - 1;
    BYTE(0xff);
  } > HEADER
}

_stack_end = __stack_origin;
_stack_start = __heap_origin;
__eheap = ORIGIN(RAM) + LENGTH(RAM);
