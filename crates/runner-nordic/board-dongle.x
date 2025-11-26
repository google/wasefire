/* nrf52840-dongle */

/* RAM */
_rstack = 0x20000008; /* 32KiB - 8B */
_rheap  = 0x20008000; /* data and heap 224KiB */
_rlimit = 0x20040000;

/* FLASH */
/* Keep in sync with the bootloader. */
_platform = 0x00008000; /* A and B 288KiB each */
_applet   = 0x00098000; /* 264KiB */
_store    = 0x000da000; /* 24KiB */
_limit    = 0x000e0000;
