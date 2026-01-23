INCLUDE board.x

ASSERT(_rstack <= _rheap, "_rstack > _rheap");
ASSERT(_rheap <= _rlimit, "_rlimit > _rheap");

ASSERT(_platform <= _applet, "_platform > _applet");
ASSERT(_applet <= _store, "_applet > _store");
ASSERT(_store <= _limit, "_store > _limit");

_platform_size = (_applet - _platform) / RUNNER_NUM_SIDES;
ASSERT(_platform_size % 0x1000 == 0, "_platform_size % 0x1000 != 0");

/* for header */
__header_origin = _platform + RUNNER_SIDE * _platform_size;
__header_length = 0x00000100; /* keep in sync with the bootloader and header library */

MEMORY {
  HEADER : ORIGIN = __header_origin, LENGTH = __header_length
  FLASH : ORIGIN = __header_origin + __header_length, LENGTH = _platform_size - __header_length
  RAM   : ORIGIN = _rheap, LENGTH = _rlimit - _rheap
}

/* for bootloader */
SECTIONS {
  .header : {
    FILL(0xffffffff); /* filled by xtask */
    . = ORIGIN(FLASH);
  } > HEADER
}

/* for runner */
__eheap = _rlimit;
__sother = _platform + (1 - RUNNER_SIDE) * _platform_size;
__eother = __sother + (RUNNER_NUM_SIDES - 1) * _platform_size;
__sapplet = _applet;
__eapplet = _store;
__sstore = _store;
__estore = _limit;

/* for cortex-m-rt */
_stack_end = _rstack;
_stack_start = _rheap;
