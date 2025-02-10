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

//! Example applet running OpenSK.

#![no_std]
wasefire::applet!();

// use alloc::boxed::Box;
// use alloc::vec::Vec;

// use opensk_lib::api::connection::HidConnection;
// use opensk_lib::api::crypto::software_crypto::SoftwareCrypto;
// use opensk_lib::api::customization::{CustomizationImpl, AAGUID_LENGTH, DEFAULT_CUSTOMIZATION};
// use opensk_lib::api::key_store;
// use opensk_lib::api::persist::{Persist, PersistIter};
// use opensk_lib::ctap::status_code::Ctap2StatusCode::{
//     self, CTAP1_ERR_OTHER, CTAP2_ERR_KEY_STORE_FULL, CTAP2_ERR_VENDOR_HARDWARE_FAILURE,
//     CTAP2_ERR_VENDOR_INTERNAL_ERROR,
// };
// use opensk_lib::ctap::status_code::CtapResult;
// use wasefire_error::{Code, Space};
// use wasefire::clock::{Handler, Timer};

mod env;

fn main() -> ! {
    todo!();
}

// pub const AAGUID: &[u8; AAGUID_LENGTH] =
//     include_bytes!(concat!(env!("OUT_DIR"), "/opensk_aaguid.bin"));

// const WASEFIRE_CUSTOMIZATION: CustomizationImpl =
//     CustomizationImpl { aaguid: AAGUID, ..DEFAULT_CUSTOMIZATION };

// #[derive(Clone, Copy, Default)]
// pub struct WasefireHidConnection;

// impl HidConnection for WasefireHidConnection {
//     fn send(
//         &mut self, buf: &[u8; 64], endpoint: opensk_lib::api::connection::UsbEndpoint,
//     ) -> CtapResult<()> {
//         todo!()
//     }

//     fn recv(
//         &mut self, buf: &mut [u8; 64], timeout_ms: usize,
//     ) -> CtapResult<opensk_lib::api::connection::RecvStatus> {
//         todo!()
//     }
// }

// fn convert(error: wasefire::Error) -> Ctap2StatusCode {
//     match (Space::try_from(error.space()), Code::try_from(error.code())) {
//         (_, Ok(Code::NotEnough)) => CTAP2_ERR_KEY_STORE_FULL,
//         (Ok(Space::User | Space::Internal), _) => CTAP2_ERR_VENDOR_INTERNAL_ERROR,
//         (Ok(Space::World), _) => CTAP2_ERR_VENDOR_HARDWARE_FAILURE,
//         _ => CTAP1_ERR_OTHER,
//     }
// }

// impl Persist for WasefireEnv {
//     fn find(&self, key: usize) -> CtapResult<Option<Vec<u8>>> {
//         match store::find(key) {
//             Err(e) => Err(convert(e)),
//             Ok(None) => Ok(None),
//             Ok(Some(data)) => Ok(Some(data.into_vec())),
//         }
//     }

//     fn insert(&mut self, key: usize, value: &[u8]) -> CtapResult<()> {
//         store::insert(key, value).map_err(convert)
//     }

//     fn remove(&mut self, key: usize) -> CtapResult<()> {
//         store::remove(key).map_err(convert)
//     }

//     fn iter(&self) -> CtapResult<PersistIter<'_>> {
//         let keys = store::keys().map_err(convert)?;
//         Ok(Box::new(keys.into_iter().map(|x| Ok(x as usize))))
//     }
// }

// impl key_store::Helper for WasefireEnv {}
