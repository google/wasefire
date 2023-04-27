// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// START OF MODULE button
// Button and touch operations.
  // Describes the state of a button.
  enum button_State {
    // The button is released.
    Released,

    // The button is pressed.
    Pressed,
  }

  // Returns how many buttons are on the device.
  @external("env", "bc")
  export declare function button_count(
  // How many buttons are on the device.
  ): usize

  // Register a handler for button events.
  @external("env", "br")
  export declare function button_register(
    // Index of the button to listen to.
    button: usize,

    // Function called on button events.
    //
    // The function takes its opaque `data` and the new button `state` as arguments.
    handler_func: usize,

    // The opaque data to use when calling the handler function.
    handler_data: usize,
  ): void

  // Unregister handlers for button events.
  @external("env", "bu")
  export declare function button_unregister(
    // Index of the button to stop listening to.
    button: usize,
  ): void
// END OF MODULE button

// START OF MODULE clock
// Clock and timer operations.
  // Whether a timer should periodically trigger.
  enum clock_Mode {
    // The timer fires only once.
    Oneshot,

    // The timer fires periodically.
    Periodic,
  }

  // Allocates a timer (initially stopped).
  @external("env", "ta")
  export declare function clock_allocate(
    // Function called when the timer triggers.
    handler_func: usize,

    // The opaque data to use when calling the handler function.
    handler_data: usize,
  // Identifier for this timer.
  ): usize

  // Starts a stopped timer given its id.
  @external("env", "tb")
  export declare function clock_start(
    // The identifier of the timer to start.
    //
    // It must come from an allocated timer that wasn't stopped.
    id: usize,

    // Whether the timer should periodically fire.
    //
    // Valid values are defined by [`Mode`](super::Mode).
    mode: usize,

    // How long until the timer triggers in milli-seconds.
    duration_ms: usize,
  ): void

  // Stops a running timer given its id.
  //
  // Note that if the timer triggers while being stopped, the handler may still be
  // called.
  @external("env", "tc")
  export declare function clock_stop(
    // The identifier of the timer to start.
    id: usize,
  ): void

  // Deallocates a stopped timer given its id.
  @external("env", "td")
  export declare function clock_free(
    // The identifier of the timer to start.
    id: usize,
  ): void
// END OF MODULE clock

// START OF MODULE crypto
// Cryptographic operations.
  // Describes errors on cryptographic operations.
  enum crypto_Error {
    // A function pre-condition was broken.
    InvalidArgument,

    // An operation is unsupported.
    Unsupported,
  }

  // START OF MODULE crypto_ccm
  // AES-CCM according to Bluetooth.
    // Whether AES-CCM is supported.
    @external("env", "ccs")
    export declare function crypto_ccm_is_supported(
    // 1 if supported, 0 otherwise.
    ): usize

    // Encrypts a clear text given a key and IV.
    @external("env", "cce")
    export declare function crypto_ccm_encrypt(
      // The 16 bytes key to encrypt with.
      key: usize,

      // The 8 bytes IV to encrypt with.
      iv: usize,

      // Length in bytes of the `clear` text.
      //
      // This must be at most 251 bytes. The `cipher` length must be 4 bytes longer than
      // this value.
      len: usize,

      // The clear text to encrypt from.
      //
      // Its length must be provided in the `len` field.
      clear: usize,

      // The cipher text to encrypt to.
      //
      // Its length must be `len + 4` bytes.
      cipher: usize,
    // Zero on success, bitwise complement of [`Error`](crate::crypto::Error)
    // otherwise.
    ): isize

    // Decrypts a cipher text given a key and IV.
    @external("env", "ccd")
    export declare function crypto_ccm_decrypt(
      // The 16 bytes key to encrypt with.
      key: usize,

      // The 8 bytes IV to encrypt with.
      iv: usize,

      // Length in bytes of the `clear` text.
      //
      // This must be at most 251 bytes. The `cipher` length must be 4 bytes longer than
      // this value.
      len: usize,

      // The cipher text to encrypt from.
      //
      // Its length must be `len + 4` bytes.
      cipher: usize,

      // The clear text to encrypt to.
      //
      // Its length must be provided in the `len` field.
      clear: usize,
    // Zero on success, bitwise complement of [`Error`](crate::crypto::Error)
    // otherwise.
    ): isize
  // END OF MODULE crypto_ccm

  // START OF MODULE crypto_gcm
  // AES-256-GCM.
    // Whether AES-256-GCM is supported.
    @external("env", "cgs")
    export declare function crypto_gcm_is_supported(
    // 1 if supported, 0 otherwise.
    ): usize

    // Encrypts and authenticates a clear text with associated data given a key and IV.
    @external("env", "cge")
    export declare function crypto_gcm_encrypt(
      // The 32 bytes key.
      key: usize,

      // The 12 bytes IV.
      iv: usize,

      // The additional authenticated data.
      aad: usize,

      // The length of the additional authenticated data.
      aad_len: usize,

      // The length of the clear (and cipher) text.
      length: usize,

      // The clear text.
      clear: usize,

      // The cipher text.
      cipher: usize,

      // The 16 bytes authentication tag.
      tag: usize,
    // Zero on success, bitwise complement of [`Error`](crate::crypto::Error)
    // otherwise.
    ): isize

    // Decrypts and authenticates a cipher text with associated data given a key and IV.
    @external("env", "cgd")
    export declare function crypto_gcm_decrypt(
      // The 32 bytes key.
      key: usize,

      // The 12 bytes IV.
      iv: usize,

      // The additional authenticated data.
      aad: usize,

      // The length of the additional authenticated data.
      aad_len: usize,

      // The 16 bytes authentication tag.
      tag: usize,

      // The length of the cipher (and clear) text.
      length: usize,

      // The cipher text.
      cipher: usize,

      // The clear text.
      clear: usize,
    // Zero on success, bitwise complement of [`Error`](crate::crypto::Error)
    // otherwise.
    ): isize
  // END OF MODULE crypto_gcm
// END OF MODULE crypto

// START OF MODULE debug
// Debugging operations.
  // Prints a message to the debug output.
  //
  // If debug output is disabled then this is a no-op.
  @external("env", "dp")
  export declare function debug_println(
    // The message to print.
    //
    // Traps if the message is not valid UTF-8.
    ptr: usize,

    // The length of the message in bytes.
    len: usize,
  ): void

  // Exits the platform with an error code.
  //
  // This is used by test applets to terminate the platform and propagate the test
  // result.
  @external("env", "de")
  export declare function debug_exit(
    // 0 for success, 1 for failure
    code: usize,
  ): void
// END OF MODULE debug

// START OF MODULE led
// LED operations.
  // Returns how many LEDs are on the device.
  @external("env", "lc")
  export declare function led_count(
  // How many LEDs are on the device.
  ): usize

  // Describes the state of a LED.
  enum led_Status {
    // The LED is off.
    Off,

    // The LED is on.
    On,
  }

  // Returns a LED status.
  @external("env", "lg")
  export declare function led_get(
    // Index of the LED to set.
    led: usize,
  // 0 for off and 1 for on.
  ): usize

  // Sets a LED status.
  @external("env", "ls")
  export declare function led_set(
    // Index of the LED to set.
    led: usize,

    // 0 for off and 1 for on.
    status: usize,
  ): void
// END OF MODULE led

// START OF MODULE rng
// Random number generators.
  // Fills a slice with random bytes.
  @external("env", "rb")
  export declare function rng_fill_bytes(
    // The slice to fill.
    ptr: usize,

    // The length of the slice.
    len: usize,
  // Error code: 0 on success, -1 on error
  //
  // The buffer may be modified on error and should not be used.
  ): isize
// END OF MODULE rng

// START OF MODULE scheduling
  // Waits until a callback is scheduled.
  //
  // This can be used as power management, since the CPU will sleep while waiting.
  @external("env", "sw")
  export declare function scheduling_wait_for_callback(
  ): void

  // Returns how many callbacks are pending.
  @external("env", "sh")
  export declare function scheduling_num_pending_callbacks(
  // How many callbacks are pending.
  ): usize
// END OF MODULE scheduling

// START OF MODULE store
// Persistent storage operations.
  // Describes errors interacting with the store.
  enum store_Error {
    // A function pre-condition was broken.
    InvalidArgument,

    // The store is full.
    NoCapacity,

    // The store reached its end of life.
    NoLifetime,

    // An operation to the underlying storage failed.
    StorageError,

    // The underlying storage doesn't match the store invariant.
    InvalidStorage,
  }

  // Inserts an entry in the store.
  //
  // If an entry for that key was already present, it is overwritten.
  @external("env", "si")
  export declare function store_insert(
    // Key of the entry.
    //
    // This must be smaller than 4096.
    key: usize,

    // Value of the entry.
    ptr: usize,

    // Length of the value.
    len: usize,
  // Zero for success. Otherwise complement of error number.
  ): isize

  // Removes an entry from the store.
  //
  // This is not an error if no entry is present. This is simply a no-op in that case.
  @external("env", "sr")
  export declare function store_remove(
    // Key of the entry.
    key: usize,
  // Zero for success. Otherwise complement of error number.
  ): isize

  // Finds an entry in the store, if any.
  @external("env", "sf")
  export declare function store_find(
    // Key of the entry to find.
    key: usize,

    // Where to write the value of the entry, if found.
    //
    // The (inner) pointer will be allocated by the callee and must be freed by the
    // caller. It is thus owned by the caller when the function returns.
    ptr: usize,

    // Where to write the length of the value, if found.
    len: usize,
  // One if found. Zero if not found. Otherwise complement of error number.
  ): isize
// END OF MODULE store

// START OF MODULE usb
// USB operations.
  // Describes errors on USB operations.
  enum usb_Error {
    Unknown,
  }

  // START OF MODULE usb_serial
    // Reads from USB serial into a buffer.
    @external("env", "usr")
    export declare function usb_serial_read(
      // Address of the buffer.
      ptr: usize,

      // Length of the buffer in bytes.
      len: usize,
    // Number of bytes read (or negative value for errors).
    //
    // This function does not block and may return zero.
    ): isize

    // Writes to USB serial from a buffer.
    @external("env", "usw")
    export declare function usb_serial_write(
      // Address of the buffer.
      ptr: usize,

      // Length of the buffer in bytes.
      len: usize,
    // Number of bytes written (or negative value for errors).
    //
    // This function does not block and may return zero.
    ): isize

    // USB serial events.
    enum usb_serial_Event {
      // Ready for read.
      Read,

      // Ready for write.
      Write,
    }

    // Registers a callback when USB serial is ready.
    //
    // It is possible that the callback is spuriously called.
    @external("env", "use")
    export declare function usb_serial_register(
      event: usize,

      handler_func: usize,

      handler_data: usize,
    ): void

    // Unregisters a callback.
    @external("env", "usd")
    export declare function usb_serial_unregister(
      event: usize,
    ): void

    // Flushs the USB serial.
    @external("env", "usf")
    export declare function usb_serial_flush(
    // Zero on success, -1 on error.
    ): isize
  // END OF MODULE usb_serial
// END OF MODULE usb

// Board-specific syscalls.
//
// Those calls are forwarded by the scheduler.
@external("env", "s")
export declare function syscall(
  x1: usize,

  x2: usize,

  x3: usize,

  x4: usize,
): isize
