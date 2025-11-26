/* nrf52840-devkit */

/* RAM */
_rstack = 0x20000000; /* 32KiB */
_rheap  = 0x20008000; /* data and heap 224KiB */
_rlimit = 0x20040000;

/* FLASH */
/* Keep in sync with the bootloader and header library. */
_platform = 0x00010000; /* A and B 336KiB each */
_applet   = 0x000b8000; /* 264KiB */
_store    = 0x000fa000; /* 24KiB */
_limit    = 0x00100000;
