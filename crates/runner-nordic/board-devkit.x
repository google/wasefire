/* nrf52840-devkit */

/* RAM */
_rstack = 0x20000000; /* 64KiB */
_rheap  = 0x20010000; /* data and heap 192KiB */
_rlimit = 0x20040000;

/* FLASH */
_platform = 0x00010000; /* A and B 320KiB each */
_applet   = 0x000b0000; /* 256KiB */
_store    = 0x000f0000; /* 64KiB */
_limit    = 0x00100000;
