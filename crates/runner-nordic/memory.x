/* nrf52840-dk

RAM:
- 0x20000000 stack (64KiB)
- 0x20010000 data and heap (192KiB)
- 0x20040000

Flash:
- 0x00000000 bootloader (64KiB)
- 0x00010000 platform A (448KiB)
- 0x00080000 platform B (448KiB)
- 0x000f0000 persistent storage (64KiB)
- 0x00100000

*/

/* Keep those values in sync with the bootloader. */
__header_origin = 0x00010000 + RUNNER_SIDE * 0x00070000;
__header_length = 0x00000100;
__flash_origin = __header_origin + __header_length;
__flash_length = 0x00070000 - __header_length;
__sother = 0x00010000 + (1 - RUNNER_SIDE) * 0x00070000;
__eother = __sother + 0x00070000;
__sstore = 0x000f0000;
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
    LONG(0xffffffff);
    LONG(0xffffffff);
    LONG(0xffffffff);
    /* We fill the unused part of the header to keep it erased. */
    FILL(0xffffffff);
    . = ORIGIN(FLASH) - 1;
    BYTE(0xff);
  } > HEADER
}

_stack_start = ORIGIN(RAM);
__eheap = ORIGIN(RAM) + LENGTH(RAM);
