/* nrf52840-dongle */

MEMORY {
  FLASH  : ORIGIN = 0x00001000, LENGTH = 0x00007000
  RUNNER : ORIGIN = 0x00008000, LENGTH = 0x00048000
  HEADER : ORIGIN = 0x00050000, LENGTH = 0x00000100
  RAM    : ORIGIN = 0x20000008, LENGTH = 0x0000fff8
}

SECTIONS {
  .runner : {
    FILL(0xffffffff);
    . = ORIGIN(RUNNER) + LENGTH(RUNNER);
  } > RUNNER

  /* Erase the header of the other runner so the bootloader won't try it. */
  .header : {
    FILL(0xffffffff);
    . = ORIGIN(HEADER) + LENGTH(HEADER);
  } > HEADER
}
