// Copyright 2023 Google LLC
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

//! Interface between the client and the device.
//!
//! The client sends requests of type `Request` and the device replies with responses of type
//! `Result<Response, String>`.

#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Request from the client to the device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    /// Generates and registers a private key for `name`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The name is too long.
    /// - The name is already registered.
    /// - The private key generation failed.
    /// - The private key registration failed (eg. not enough storage).
    Register { name: String },

    /// Signs `challenge` using the private key for `name`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The name is not registered.
    /// - The signature failed.
    Authenticate { name: String, challenge: [u8; 32] },

    /// Lists all registered names.
    ///
    /// # Errors
    ///
    /// Returns an error if the listing failed.
    List,

    /// Deletes the private key for `name`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The name is not registered.
    /// - The deletion failed.
    Delete { name: String },
}

/// Response from the device to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// Returns the public key of the generated private key.
    Register { x: [u8; 32], y: [u8; 32] },

    /// Returns the signature of the challenge.
    Authenticate { r: [u8; 32], s: [u8; 32] },

    /// Returns the list of registered names.
    List { names: Vec<String> },

    /// Does not return anything.
    Delete,
}

/// Reads until an object is deserialized.
///
/// The `read` function parameter must block until at least one byte can be read. Then, it should
/// read as much as possible without blocking and return that length.
pub fn deserialize<T: DeserializeOwned>(mut read: impl FnMut(&mut [u8]) -> usize) -> T {
    let mut buffer = Vec::new();
    loop {
        const SIZE: usize = 32;
        let len = buffer.len();
        buffer.resize(len + SIZE, 0);
        let read = read(&mut buffer[len ..]);
        buffer.truncate(len + read);
        if buffer[len ..].iter().any(|&x| x == 0) {
            break;
        }
    }
    let len = buffer.len();
    assert_eq!(buffer[len - 1], 0);
    assert!(buffer[.. len - 1].iter().all(|&x| x != 0));
    postcard::from_bytes_cobs(&mut buffer).unwrap()
}

/// Serializes an object.
pub fn serialize<T: Serialize>(x: &T) -> Vec<u8> {
    postcard::to_allocvec_cobs(x).unwrap()
}
