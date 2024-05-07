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
    Released = 0,

    // The button is pressed.
    Pressed = 1,
  }

  // Returns how many buttons are on the device.
  @external("env", "bc")
  export declare function button_count(
  ): i32

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
  ): i32

  // Unregister handlers for button events.
  @external("env", "bu")
  export declare function button_unregister(
    // Index of the button to stop listening to.
    button: usize,
  ): i32
// END OF MODULE button

// START OF MODULE crypto
// Cryptographic operations.
  // START OF MODULE crypto_ccm
  // AES-CCM according to Bluetooth.
    // Whether AES-CCM is supported.
    @external("env", "ccs")
    export declare function crypto_ccm_is_supported(
    ): i32

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
    ): i32

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
    ): i32
  // END OF MODULE crypto_ccm

  // START OF MODULE crypto_ec
  // Elliptic curves.
    enum crypto_ec_Curve {
      P256 = 0,

      P384 = 1,
    }

    // Whether a curve is supported.
    @external("env", "ces")
    export declare function crypto_ec_is_supported(
      // The enum value of the [curve][super::Curve].
      curve: usize,
    ): i32

    // Returns whether a scalar is valid.
    //
    // A scalar is valid if smaller than the field's modulus.
    @external("env", "cet")
    export declare function crypto_ec_is_valid_scalar(
      // The curve.
      curve: usize,

      // The scalar in SEC1 encoding.
      n: usize,
    ): i32

    // Returns whether a point is valid.
    @external("env", "ceq")
    export declare function crypto_ec_is_valid_point(
      // The curve.
      curve: usize,

      // The x-coordinate in SEC1 encoding.
      x: usize,

      // The y-coordinate in SEC1 encoding.
      y: usize,
    ): i32

    // Performs base point multiplication.
    @external("env", "ceb")
    export declare function crypto_ec_base_point_mul(
      // The curve.
      curve: usize,

      // The scalar in SEC1 encoding.
      n: usize,

      // The x coordinate in SEC1 encoding.
      x: usize,

      // The y coordinate in SEC1 encoding.
      y: usize,
    ): i32

    // Performs point multiplication.
    @external("env", "cep")
    export declare function crypto_ec_point_mul(
      // The curve.
      curve: usize,

      // The scalar in SEC1 encoding.
      n: usize,

      // The x coordinate of the input point in SEC1 encoding.
      in_x: usize,

      // The y coordinate of the input point in SEC1 encoding.
      in_y: usize,

      // The x coordinate of the output point in SEC1 encoding.
      out_x: usize,

      // The y coordinate of the output point in SEC1 encoding.
      out_y: usize,
    ): i32

    // Signs a message with ECDSA.
    @external("env", "cei")
    export declare function crypto_ec_ecdsa_sign(
      // The curve.
      curve: usize,

      // The private key scalar in SEC1 encoding.
      key: usize,

      // The integer message in SEC1 encoding.
      message: usize,

      // The r signature component in SEC1 encoding.
      r: usize,

      // The s signature component in SEC1 encoding.
      s: usize,
    ): i32

    // Verifies an ECDSA signature.
    @external("env", "cev")
    export declare function crypto_ec_ecdsa_verify(
      // The curve.
      curve: usize,

      // The integer message in SEC1 encoding.
      message: usize,

      // The x-coordinate in SEC1 encoding.
      x: usize,

      // The y-coordinate in SEC1 encoding.
      y: usize,

      // The r signature component in SEC1 encoding.
      r: usize,

      // The s signature component in SEC1 encoding.
      s: usize,
    ): i32
  // END OF MODULE crypto_ec

  // START OF MODULE crypto_gcm
  // AES-256-GCM.
    // Bit-shift for the supported bit-flags.
    enum crypto_gcm_Support {
      // The [`encrypt()`] and [`decrypt()`] functions are supported without copy when
      // the input pointer is non-null, i.e. the function uses different buffers for
      // input and output.
      NoCopy = 0,

      // The [`encrypt()`] and [`decrypt()`] functions are supported without copy when
      // the input pointer is null, i.e. the function operates in-place in the same
      // buffer.
      InPlaceNoCopy = 1,
    }

    // Describes how AES-256-GCM is supported.
    //
    // Returns a bit-flag described by [`super::Support`] on success.
    @external("env", "cgs")
    export declare function crypto_gcm_support(
    ): i32

    // Returns the supported tag length.
    //
    // The tag argument to [`encrypt()`] and [`decrypt()`] must be of that length.
    @external("env", "cgt")
    export declare function crypto_gcm_tag_length(
    ): i32

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
      //
      // A null pointer indicates that the clear text is in the cipher text and should be
      // encrypted in place.
      clear: usize,

      // The cipher text.
      cipher: usize,

      // The authentication tag (see [`super::tag_length()`]).
      tag: usize,
    ): i32

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

      // The authentication tag (see [`super::tag_length()`]).
      tag: usize,

      // The length of the cipher (and clear) text.
      length: usize,

      // The cipher text.
      //
      // A null pointer indicates that the cipher text is in the clear text and should be
      // decrypted in place.
      cipher: usize,

      // The clear text.
      clear: usize,
    ): i32
  // END OF MODULE crypto_gcm

  // START OF MODULE crypto_hash
  // Hash functions.
    // Hash algorithm.
    enum crypto_hash_Algorithm {
      // SHA-256.
      Sha256 = 0,

      // SHA-384.
      Sha384 = 1,
    }

    // Whether the algorithm is supported.
    @external("env", "chs")
    export declare function crypto_hash_is_supported(
      // The hash algorithm.
      algorithm: usize,
    ): i32

    // Initializes a hash and returns its identifier.
    @external("env", "chi")
    export declare function crypto_hash_initialize(
      // The hash algorithm.
      algorithm: usize,
    ): i32

    // Updates a hash.
    //
    // Errors are surfaced in the [`finalize()`] call.
    @external("env", "chu")
    export declare function crypto_hash_update(
      // The identifier returned by the associated [`initialize()`] call.
      id: usize,

      // The pointer to the data to hash.
      data: usize,

      // The length of the data to hash.
      length: usize,
    ): i32

    // Finalizes a hash.
    @external("env", "chf")
    export declare function crypto_hash_finalize(
      // The identifier returned by the associated [`initialize()`] call.
      //
      // This is consumed and invalidated by this call regardless of the return value.
      id: usize,

      // The pointer to the buffer where the digest must be written.
      //
      // Its length is defined by the algorithm:
      // - 32 bytes for SHA-256.
      //
      // The pointer may be null, in which case this function deallocates the identifier
      // without computing the digest.
      digest: usize,
    ): i32

    // Whether the algorithm is supported for hmac.
    @external("env", "cht")
    export declare function crypto_hash_is_hmac_supported(
      // The hash algorithm.
      algorithm: usize,
    ): i32

    // Initializes an hmac and returns its identifier.
    @external("env", "chj")
    export declare function crypto_hash_hmac_initialize(
      // The hash algorithm.
      algorithm: usize,

      // The pointer to the key.
      key: usize,

      // The length of the key.
      //
      // If greater than 64 bytes, the key will be itself hashed.
      key_len: usize,
    ): i32

    // Updates an hmac.
    //
    // Errors are surfaced in the [`hmac_finalize()`] call.
    @external("env", "chv")
    export declare function crypto_hash_hmac_update(
      // The identifier returned by the associated [`hmac_initialize()`] call.
      id: usize,

      // The pointer to the data to hmac.
      data: usize,

      // The length of the data to hmac.
      length: usize,
    ): i32

    // Finalizes an hmac.
    @external("env", "chg")
    export declare function crypto_hash_hmac_finalize(
      // The identifier returned by the associated [`hmac_initialize()`] call.
      //
      // This is consumed and invalidated by this call regardless of the return value.
      id: usize,

      // The pointer to the buffer where the hmac must be written.
      //
      // Its length is defined by the algorithm:
      // - 32 bytes for SHA-256.
      //
      // The pointer may be null, in which case this function deallocates the identifier
      // without computing the hmac.
      hmac: usize,
    ): i32

    // Whether the algorithm is supported for hkdf.
    @external("env", "chr")
    export declare function crypto_hash_is_hkdf_supported(
      // The hash algorithm.
      algorithm: usize,
    ): i32

    // Expands with RFC5869 HKDF.
    @external("env", "che")
    export declare function crypto_hash_hkdf_expand(
      // The hash algorithm.
      algorithm: usize,

      // The pointer to the pseudo random key.
      prk: usize,

      // The length of the pseudo random key.
      //
      // Must be at least the length of the hash algorithm output.
      prk_len: usize,

      // The pointer to the info.
      //
      // May be null if [`info_len`] is null.
      info: usize,

      // The length of the info.
      //
      // May be zero.
      info_len: usize,

      // The pointer to the output key material.
      okm: usize,

      // The length of the output key material.
      //
      // Must be at most 255 times the output length of the hash algorithm.
      okm_len: usize,
    ): i32
  // END OF MODULE crypto_hash
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
  ): i32

  // Returns the time spent since some initial event.
  //
  // The time is in micro-seconds and may wrap before using all 64 bits.
  @external("env", "dt")
  export declare function debug_time(
    // Pointer to the 64-bits time (may be null).
    //
    // The least significant 31 bits are always returned, regardless of whether the
    // pointer is null.
    ptr: usize,
  ): i32

  // Time in micro-seconds since the scheduler started.
  //
  // Values may not be accurate for a few reasons:
  // - Interrupts are accounted to the component they interrupt. Ideally it should be
  // accounted to the platform only.
  // - Allocation in the applet by the platform is accounted to the platform instead of
  // the applet.
  // - The board time is assumed to have a longer wrap period than any continuous
  // component run time.
  class debug_Perf {
    // Time spent in the platform.
    platform: u64;

    // Time spent in applets.
    applets: u64;

    // Time spent waiting for events.
    waiting: u64;
  }

  // Returns the time spent since some initial event, split by component.
  @external("env", "dq")
  export declare function debug_perf(
    // Pointer to the output [`super::Perf`] struct.
    ptr: usize,
  ): i32

  // Exits the platform with an error code.
  //
  // This is used by test applets to terminate the platform and propagate the test
  // result.
  @external("env", "de")
  export declare function debug_exit(
    // 0 for success, 1 for failure.
    code: usize,
  ): i32
// END OF MODULE debug

// START OF MODULE gpio
// Low-level GPIO operations.
//
// See [`crate::button`] and [`crate::led`] for higher-level GPIO operations.
  // Returns how many GPIOs are on the device.
  @external("env", "gc")
  export declare function gpio_count(
  ): i32

  // Input configuration.
  enum gpio_InputConfig {
    // Input is disabled.
    Disabled = 0,

    // Floating input (most common configuration).
    //
    // Reading while the voltage is not driven may return 0 or 1.
    Floating = 1,

    // Pull-down input.
    //
    // Reading while the voltage is not driven returns 0.
    PullDown = 2,

    // Pull-up input.
    //
    // Reading while the voltage is not driven returns 1.
    PullUp = 3,
  }

  // Output configuration.
  enum gpio_OutputConfig {
    // Output is disabled.
    Disabled = 0,

    // Push-pull output (most common configuration).
    //
    // Writing 0 (resp. 1) drives the voltage to 0 (resp. 1).
    PushPull = 3,

    // Open-drain output.
    //
    // Writing 0 drives the voltage to 0. Writing 1 doesn't drive the voltage.
    OpenDrain = 1,

    // Open-source output.
    //
    // Writing 0 doesn't drive the voltage. Writing 1 drives the voltage to 1.
    OpenSource = 2,
  }

  // Configures a GPIO.
  @external("env", "gf")
  export declare function gpio_configure(
    // Index of the GPIO to configure.
    gpio: usize,

    // Bit-field describing the configuration.
    //
    // | Bits  | Description          |
    // | ---   | ---                  |
    // | 01:00 | Input configuration  |
    // | 09:08 | Output configuration |
    // | 16:16 | Output initial value |
    mode: usize,
  ): i32

  // Reads from a GPIO.
  @external("env", "gr")
  export declare function gpio_read(
    // Index of the GPIO to read from (must be configured as input).
    gpio: usize,
  ): i32

  // Writes to a GPIO.
  @external("env", "gw")
  export declare function gpio_write(
    // Index of the GPIO to write to (must be configured as output).
    gpio: usize,

    // Logical value (0 or 1).
    val: usize,
  ): i32

  // Returns the last logical value written to a GPIO.
  //
  // The initial output value counts as a write and would be returned if `write()` was
  // not called since last `configure()`.
  @external("env", "gl")
  export declare function gpio_last_write(
    // Index of the GPIO to query (must be configured as output).
    gpio: usize,
  ): i32
// END OF MODULE gpio

// START OF MODULE led
// LED operations.
  // Returns how many LEDs are on the device.
  @external("env", "lc")
  export declare function led_count(
  ): i32

  // Describes the state of a LED.
  enum led_Status {
    // The LED is off.
    Off = 0,

    // The LED is on.
    On = 1,
  }

  // Returns whether a LED is on.
  @external("env", "lg")
  export declare function led_get(
    // Index of the LED to set.
    led: usize,
  ): i32

  // Sets a LED status.
  @external("env", "ls")
  export declare function led_set(
    // Index of the LED to set.
    led: usize,

    // 0 for off and 1 for on.
    status: usize,
  ): i32
// END OF MODULE led

// START OF MODULE platform
// Platform operations.
  // START OF MODULE platform_protocol
  // Platform protocol.
    // Reads the last request, if any.
    //
    // Returns whether a request was allocated.
    @external("env", "ppr")
    export declare function platform_protocol_read(
      // Where to write the request, if any.
      //
      // The (inner) pointer will be allocated by the callee and must be freed by the
      // caller. It is thus owned by the caller when the function returns.
      ptr: usize,

      // Where to write the length of the request, if any.
      len: usize,
    ): i32

    // Writes a response to the last request.
    @external("env", "ppw")
    export declare function platform_protocol_write(
      // Address of the response.
      ptr: usize,

      // Length of the response in bytes.
      len: usize,
    ): i32

    // Registers a callback when a request is received.
    @external("env", "ppe")
    export declare function platform_protocol_register(
      handler_func: usize,

      handler_data: usize,
    ): i32

    // Unregisters the callback.
    @external("env", "ppd")
    export declare function platform_protocol_unregister(
    ): i32
  // END OF MODULE platform_protocol

  // START OF MODULE platform_update
  // Operations to update the platform.
  //
  // All operations are abstract over the update content such that they can work on all
  // platforms. In particular, chunks and errors are platform-specific. Applets with
  // knowledge about their platform may actually inspect that content for additional checks.
    // Whether platform update is supported.
    @external("env", "pus")
    export declare function platform_update_is_supported(
    ): i32

    // Returns the metadata of the platform.
    //
    // This typically contains the version and side (A or B) of the running platform.
    @external("env", "pum")
    export declare function platform_update_metadata(
      // Where to write the allocated metadata.
      ptr: usize,

      // Where to write the metadata length.
      len: usize,
    ): i32

    // Starts a platform update process.
    @external("env", "pui")
    export declare function platform_update_initialize(
      // Zero for normal operation. One for dry-run.
      //
      // During a dry-run, any mutable operation is skipped and only checks are
      // performed.
      dry_run: usize,
    ): i32

    // Processes the next chunk of a platform update.
    @external("env", "pup")
    export declare function platform_update_process(
      // Address of the chunk.
      ptr: usize,

      // Length of the chunk in bytes.
      len: usize,
    ): i32

    // Finalizes a platform update process.
    //
    // This function will reboot when the update is successful and thus only returns in
    // case of errors or in dry-run mode.
    @external("env", "puf")
    export declare function platform_update_finalize(
    ): i32
  // END OF MODULE platform_update

  // Returns the serial of the platform.
  //
  // Returns the length of the serial in bytes. The serial is not allocated if the
  // length is zero (and the pointer is not written).
  @external("env", "ps")
  export declare function platform_serial(
    // Where to write the serial.
    //
    // If the returned length is positive, the (inner) pointer will be allocated by the
    // callee and must be freed by the caller. It is thus owned by the caller when the
    // function returns.
    ptr: usize,
  ): i32

  // Returns the version of the platform.
  //
  // Returns the length of the version in bytes. The version is not allocated if the
  // length is zero (and the pointer is not written).
  @external("env", "pv")
  export declare function platform_version(
    // Where to write the version.
    //
    // If the returned length is positive, the (inner) pointer will be allocated by the
    // callee and must be freed by the caller. It is thus owned by the caller when the
    // function returns.
    ptr: usize,
  ): i32

  // Reboots the device (thus platform and applets).
  //
  // Does not return on success.
  @external("env", "pr")
  export declare function platform_reboot(
  ): i32
// END OF MODULE platform

// START OF MODULE radio
// Radio operations.
  // START OF MODULE radio_ble
  // Bluetooth Low Energy (BLE) operations.
    // BLE events.
    enum radio_ble_Event {
      // Advertisement packets.
      Advertisement = 0,
    }

    // Advertisement packet.
    class radio_ble_Advertisement {
      ticks: u32;

      freq: u16;

      rssi: i8;

      pdu_type: u8;

      addr: unimplemented;

      data_len: u8;

      data: unimplemented;

      _padding: unimplemented;
    }

    // Reads the next advertisement packet into a buffer, if any.
    //
    // Returns whether a packet was read.
    @external("env", "rlra")
    export declare function radio_ble_read_advertisement(
      // Pointer to the [`super::Advertisement`] packet.
      ptr: usize,
    ): i32

    // Register a handler for radio events.
    @external("env", "rle")
    export declare function radio_ble_register(
      // Radio [`super::Event`] to listen to.
      event: u32,

      // Function called on radio events.
      //
      // The function takes its opaque `data` as argument.
      handler_func: usize,

      // The opaque data to use when calling the handler function.
      handler_data: usize,
    ): i32

    // Unregister handlers for radio events.
    @external("env", "rld")
    export declare function radio_ble_unregister(
      // Radio [`super::Event`] to stop listening to.
      event: u32,
    ): i32
  // END OF MODULE radio_ble
// END OF MODULE radio

// START OF MODULE rng
// Random number generators.
  // Fills a slice with random bytes.
  @external("env", "rb")
  export declare function rng_fill_bytes(
    // The slice to fill.
    ptr: usize,

    // The length of the slice.
    len: usize,
  ): i32
// END OF MODULE rng

// START OF MODULE scheduling
  // Waits until a callback is scheduled.
  //
  // This can be used as power management, since the CPU will sleep while waiting.
  @external("env", "sw")
  export declare function scheduling_wait_for_callback(
  ): i32

  // Returns how many callbacks are pending.
  @external("env", "sh")
  export declare function scheduling_num_pending_callbacks(
  ): i32

  // Aborts the applet.
  @external("env", "sa")
  export declare function scheduling_abort(
  ): i32
// END OF MODULE scheduling

// START OF MODULE store
// Persistent storage operations.
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
  ): i32

  // Removes an entry from the store.
  //
  // This is not an error if no entry is present. This is simply a no-op in that case.
  @external("env", "sr")
  export declare function store_remove(
    // Key of the entry.
    key: usize,
  ): i32

  // Finds an entry in the store, if any.
  //
  // Returns whether an entry was found.
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
  ): i32

  // Returns the unordered keys of the entries in the store.
  //
  // Returns the number of keys, and thus the length of the array. The array is not
  // allocated if the length is zero (and the pointer is not written).
  @external("env", "sk")
  export declare function store_keys(
    // Where to write the keys as an array of u16, if at least one.
    //
    // The (inner) pointer will be allocated by the callee and must be freed by the
    // caller. It is thus owned by the caller when the function returns.
    ptr: usize,
  ): i32

  // Clears the store, removing all entries.
  @external("env", "sc")
  export declare function store_clear(
  ): i32

  // START OF MODULE store_fragment
  // Support for fragmented entries.
    // Inserts an entry in the store.
    //
    // The entry will be fragmented over multiple keys within the provided range as needed.
    //
    // If an entry for that range of keys was already present, it is overwritten.
    @external("env", "sfi")
    export declare function store_fragment_insert(
      // Range of keys where to insert the fragments.
      //
      // This is a pair of u16: the lowest u16 is the first key of the range and the
      // highest u16 is one past the last key of the range.
      keys: u32,

      // Value of the entry.
      ptr: usize,

      // Length of the value.
      len: usize,
    ): i32

    // Removes an entry from the store.
    //
    // All fragments from the range of keys will be deleted.
    //
    // This is not an error if no entry is present. This is simply a no-op in that case.
    @external("env", "sfr")
    export declare function store_fragment_remove(
      // Range of keys to remove.
      keys: u32,
    ): i32

    // Finds an entry in the store, if any.
    //
    // The entry may be fragmented withen the provided range.
    //
    // Returns whether an entry was found.
    @external("env", "sff")
    export declare function store_fragment_find(
      // Range of keys to concatenate as an entry.
      keys: u32,

      // Where to write the value of the entry, if found.
      //
      // The (inner) pointer will be allocated by the callee and must be freed by the
      // caller. It is thus owned by the caller when the function returns.
      ptr: usize,

      // Where to write the length of the value, if found.
      len: usize,
    ): i32
  // END OF MODULE store_fragment
// END OF MODULE store

// START OF MODULE timer
// Timer operations.
  // Whether a timer should periodically trigger.
  enum timer_Mode {
    // The timer fires only once.
    Oneshot = 0,

    // The timer fires periodically.
    Periodic = 1,
  }

  // Allocates a timer (initially stopped) and returns its identifier.
  @external("env", "ta")
  export declare function timer_allocate(
    // Function called when the timer triggers.
    handler_func: usize,

    // The opaque data to use when calling the handler function.
    handler_data: usize,
  ): i32

  // Starts a stopped timer given its identifier.
  @external("env", "tb")
  export declare function timer_start(
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
  ): i32

  // Stops a running timer given its identifier.
  //
  // Note that if the timer triggers while being stopped, the handler may still be
  // called.
  @external("env", "tc")
  export declare function timer_stop(
    // The identifier of the timer to start.
    id: usize,
  ): i32

  // Deallocates a stopped timer given its identifier.
  @external("env", "td")
  export declare function timer_free(
    // The identifier of the timer to start.
    id: usize,
  ): i32
// END OF MODULE timer

// START OF MODULE uart
  // Returns how many UARTs are on the device.
  @external("env", "uac")
  export declare function uart_count(
  ): i32

  // Sets the baudrate of a stopped UART.
  @external("env", "uaz")
  export declare function uart_set_baudrate(
    // Index of the UART to configure.
    uart: usize,

    // Baudrate to configure.
    baudrate: usize,
  ): i32

  // Starts a UART.
  @external("env", "uaa")
  export declare function uart_start(
    // Index of the UART to start.
    uart: usize,
  ): i32

  // Stops a UART.
  @external("env", "uab")
  export declare function uart_stop(
    // Index of the UART to stop.
    uart: usize,
  ): i32

  // Reads from a UART into a buffer.
  //
  // Returns the number of bytes read. This function does not block and may return zero.
  @external("env", "uar")
  export declare function uart_read(
    // Index of the UART to read from.
    uart: usize,

    // Address of the buffer.
    ptr: usize,

    // Length of the buffer in bytes.
    len: usize,
  ): i32

  // Writes to a UART from a buffer.
  //
  // Returns the number of bytes written. This function does not block and may return
  // zero.
  @external("env", "uaw")
  export declare function uart_write(
    // Index of the UART to write to.
    uart: usize,

    // Address of the buffer.
    ptr: usize,

    // Length of the buffer in bytes.
    len: usize,
  ): i32

  // UART events.
  enum uart_Event {
    // Ready for read.
    Read = 0,

    // Ready for write.
    Write = 1,
  }

  // Registers a callback when a UART is ready.
  //
  // It is possible that the callback is spuriously called. The callback is only
  // guaranteed to be called after the associated operation processed less bytes than the
  // buffer size.
  @external("env", "uae")
  export declare function uart_register(
    // Index of the UART to listen to.
    uart: usize,

    // Event to listen to.
    event: usize,

    // Function pointer of the closure to call on events.
    handler_func: usize,

    // Opaque data of the closure to call on events.
    handler_data: usize,
  ): i32

  // Unregisters a callback.
  @external("env", "uad")
  export declare function uart_unregister(
    // Index of the UART to stop listening to.
    uart: usize,

    // Event to stop listening to.
    event: usize,
  ): i32
// END OF MODULE uart

// START OF MODULE usb
// USB operations.
  // START OF MODULE usb_serial
    // Reads from USB serial into a buffer.
    //
    // Returns the number of bytes read. This function does not block and may return zero.
    @external("env", "usr")
    export declare function usb_serial_read(
      // Address of the buffer.
      ptr: usize,

      // Length of the buffer in bytes.
      len: usize,
    ): i32

    // Writes to USB serial from a buffer.
    //
    // Returns the number of bytes written. This function does not block and may return
    // zero.
    @external("env", "usw")
    export declare function usb_serial_write(
      // Address of the buffer.
      ptr: usize,

      // Length of the buffer in bytes.
      len: usize,
    ): i32

    // USB serial events.
    enum usb_serial_Event {
      // Ready for read.
      Read = 0,

      // Ready for write.
      Write = 1,
    }

    // Registers a callback when USB serial is ready.
    //
    // It is possible that the callback is spuriously called. The callback is only
    // guaranteed to be called after the associated operation processed less bytes than the
    // buffer size.
    @external("env", "use")
    export declare function usb_serial_register(
      event: usize,

      handler_func: usize,

      handler_data: usize,
    ): i32

    // Unregisters a callback.
    @external("env", "usd")
    export declare function usb_serial_unregister(
      event: usize,
    ): i32

    // Flushs the USB serial.
    @external("env", "usf")
    export declare function usb_serial_flush(
    ): i32
  // END OF MODULE usb_serial
// END OF MODULE usb

// Board-specific syscalls.
//
// Those calls are directly forwarded to the board by the scheduler.
@external("env", "s")
export declare function syscall(
  x1: usize,

  x2: usize,

  x3: usize,

  x4: usize,
): i32
