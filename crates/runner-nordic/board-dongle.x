/* nrf52840-dongle */

/* RAM */
_rstack = 0x20000008; /* 64KiB - 8B */
_rheap  = 0x20010000; /* data and heap 192KiB */
_rlimit = 0x20040000;

/* FLASH */
_platform = 0x00008000; /* A and B 288KiB each */
_applet   = 0x00098000; /* 256KiB */
_store    = 0x000d8000; /* 32KiB */
_limit    = 0x000e0000;
