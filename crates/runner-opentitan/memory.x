/* RAM */
_rcritic = 0x10000000; /* static critical (8 KiB) */
_rdata   = 0x10002000; /* 8 KiB + 64KiB in wasm */
_rheap   = 0x10014000 - RUNNER_NATIVE * 0x10000; /* 32 KiB + 64KiB in native */
_rstack  = 0x1001c000; /* 16 KiB - 128 B */
_rlimit  = 0x1001ff80; /* exception frame (128 B) */

/* FLASH
The memory protections RWESEHL are read, write, erase, scramble, error-correction, high-endurance,
and lock respectively. We modify the configuration for the storage to remove error-correction since
we need to write multiple times. The regions 0 to 3 are read-only for the active side, otherwise can
be written and erased.

0 R--SE-L 0x20000000
2 R--SE-L 0x20010000
3 RWE--H- 0x20077000
1 RWESE-L 0x20080000
4 RWESE-L 0x20090000
5 RWE--H- 0x200f7000
6 R--SE-L 0x200ff800
*/
_rom_ext  = 0x20000000; /* 64 KiB */
_manifest = 0x20010000; /* 1 KiB */
_platform = 0x20010400; /* 351 KiB */
_appleta  = 0x20068000; /* 60 KiB */
_storea   = 0x20077000; /* 36 KiB */
_limita   = 0x20080000;
_appletb  = 0x200e8000; /* 60 KiB */
_storeb   = 0x200f7000; /* 34 KiB */
_limitb   = 0x200ff800;
ASSERT(_appleta + 0x80000 == _appletb, "Bad applet region");
ASSERT(_storea + 0x80000 == _storeb, "Bad store region");

BANK_OFF = RUNNER_SIDE * 0x80000;
MEMORY {
  MANIF  (R) : ORIGIN = _manifest + BANK_OFF, LENGTH = _platform - _manifest
  FLASH (RX) : ORIGIN = _platform + BANK_OFF, LENGTH = _appleta - _platform
  DATA   (W) : ORIGIN = _rdata, LENGTH = _rheap - _rdata
  HEAP   (W) : ORIGIN = _rheap, LENGTH = _rstack - _rheap
  STACK  (W) : ORIGIN = _rstack, LENGTH = _rlimit - _rstack
}

SECTIONS {
  .manifest : {
    _smanifest = .;
    . = ORIGIN(MANIF) + 816;
    LONG(0x1d4); /* address translation */
    . = ORIGIN(MANIF) + 824;
    LONG(0x71c36c47); /* manifest version */
    . = ORIGIN(MANIF) + 892;
    LONG(ADDR(.text) - ORIGIN(MANIF)); /* code start */
    LONG(ADDR(.text) + SIZEOF(.text) - ORIGIN(MANIF)); /* code end */
    LONG(_start - ORIGIN(MANIF)); /* entry point */
    . = ORIGIN(MANIF) + LENGTH(MANIF);
  } > MANIF
}

/* riscv-rt configuration */
REGION_ALIAS("REGION_TEXT", FLASH);
REGION_ALIAS("REGION_RODATA", FLASH);
REGION_ALIAS("REGION_DATA", DATA);
REGION_ALIAS("REGION_BSS", DATA);
REGION_ALIAS("REGION_HEAP", HEAP);
REGION_ALIAS("REGION_STACK", STACK);
_heap_size = LENGTH(HEAP);
