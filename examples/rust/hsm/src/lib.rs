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

//! Implements some HSM-like API.
//!
//! The applet combines crypto, store, and USB serial. It comes with a companion program in the
//! `host` directory because the USB serial channel is binary (data is serialized and deserialized).
//! The companion program is self-documented with `cargo run -- help`.

#![no_std]
wasefire::applet!();

use common::{Deserialize, Deserializer, Error, Request, Response, Serialize, Serializer};

fn main() {
    loop {
        write(&process(read()));
    }
}

fn process(request: Request) -> Result<Response, Error> {
    match request {
        Request::GenerateKey { key } => {
            let mut secret = [0; 16];
            rng::fill_bytes(&mut secret)?;
            store::insert(key, &secret)?;
            Ok(Response::GenerateKey)
        }
        Request::DeleteKey { key } => {
            store::remove(key)?;
            Ok(Response::DeleteKey)
        }
        Request::Encrypt { key, nonce, data } => {
            let key = store::find(key)?.ok_or(Error::BadHandle)?;
            let key = <&[u8] as TryInto<_>>::try_into(&key).unwrap();
            let data = crypto::ccm::encrypt(key, &nonce, &data)?;
            Ok(Response::Encrypt { data })
        }
        Request::Decrypt { key, nonce, data } => {
            let key = store::find(key)?.ok_or(Error::BadHandle)?;
            let key = <&[u8] as TryInto<_>>::try_into(&key).unwrap();
            let data = crypto::ccm::decrypt(key, &nonce, &data)?;
            Ok(Response::Decrypt { data })
        }
        Request::ImportKey { key, secret } => {
            store::insert(key, &secret)?;
            Ok(Response::ImportKey)
        }
        Request::ExportKey { key } => {
            let secret = store::find(key)?.ok_or(Error::BadHandle)?;
            let secret = <&[u8] as TryInto<_>>::try_into(&secret).unwrap();
            Ok(Response::ExportKey { secret })
        }
    }
}

fn read<T: Deserialize>() -> T {
    T::deserialize(&mut Usb).unwrap()
}

fn write<T: Serialize>(x: &T) {
    x.serialize(&mut Usb).unwrap()
}

struct Usb;

impl Deserializer for Usb {
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Error> {
        usb::serial::read_all(data).map_err(|_| Error::UsbError)
    }
}

impl Serializer for Usb {
    fn write_all(&mut self, data: &[u8]) -> Result<(), Error> {
        usb::serial::write_all(data).map_err(|_| Error::UsbError)
    }

    fn flush(&mut self) -> Result<(), Error> {
        usb::serial::flush().map_err(|_| Error::UsbError)
    }
}
