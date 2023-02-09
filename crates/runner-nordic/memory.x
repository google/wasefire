/* nrf52840-dk */

__stack_size = 0x10000;
__store_size = 0x10000;

MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 0x00100000 - __store_size
  RAM   : ORIGIN = 0x20000000 + __stack_size, LENGTH = 0x00040000 - __stack_size
}

_stack_start = ORIGIN(RAM);
__eheap = ORIGIN(RAM) + LENGTH(RAM);
__sstore = ORIGIN(FLASH) + LENGTH(FLASH);
__estore = __sstore + __store_size;
