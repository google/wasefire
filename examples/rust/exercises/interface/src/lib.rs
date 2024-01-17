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

/// Deserializes an object.
///
/// Taking the buffer as mutable is a quirk from postcard.
pub fn deserialize<T: DeserializeOwned>(x: &mut [u8]) -> T {
    postcard::from_bytes_cobs(x).unwrap()
}

/// Serializes an object.
pub fn serialize<T: Serialize>(x: &T) -> Vec<u8> {
    postcard::to_allocvec_cobs(x).unwrap()
}
