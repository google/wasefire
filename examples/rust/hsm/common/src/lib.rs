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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

pub type KeyHandle = usize;

#[derive(Debug)]
pub enum Request {
    GenerateKey { key: KeyHandle },
    DeleteKey { key: KeyHandle },
    Encrypt { key: KeyHandle, nonce: [u8; 8], data: Vec<u8> },
    Decrypt { key: KeyHandle, nonce: [u8; 8], data: Vec<u8> },
    ImportKey { key: KeyHandle, secret: [u8; 16] },
    ExportKey { key: KeyHandle },
}

#[derive(Debug)]
pub enum Response {
    GenerateKey,
    DeleteKey,
    Encrypt { data: Vec<u8> },
    Decrypt { data: Vec<u8> },
    ImportKey,
    ExportKey { secret: [u8; 16] },
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum Error {
    #[cfg_attr(feature = "std", error("unknown"))]
    Unknown,
    #[cfg_attr(feature = "std", error("bad handle"))]
    BadHandle,
    #[cfg_attr(feature = "std", error("bad format"))]
    BadFormat,
    #[cfg_attr(feature = "std", error("crypto error"))]
    CryptoError,
    #[cfg_attr(feature = "std", error("rng error"))]
    RngError,
    #[cfg_attr(feature = "std", error("store error"))]
    StoreError,
    #[cfg_attr(feature = "std", error("usb error"))]
    UsbError,
}

pub trait Serializer {
    fn write_all(&mut self, data: &[u8]) -> Result<(), Error>;
    fn flush(&mut self) -> Result<(), Error>;
}

pub trait Serialize {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), Error>;
}

pub trait Deserializer {
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Error>;
}

pub trait Deserialize: Sized {
    fn deserialize<S: Deserializer>(deserializer: &mut S) -> Result<Self, Error>;
}

impl Serialize for usize {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), Error> {
        let mut input = *self;
        let mut output = Vec::new();
        let mut chunk = 0x80;
        while chunk & 0x80 != 0 {
            chunk = (input & 0x7f) as u8;
            input >>= 7;
            if input > 0 {
                chunk |= 0x80;
            }
            output.push(chunk);
        }
        serializer.write_all(&output)
    }
}

impl Deserialize for usize {
    fn deserialize<S: Deserializer>(deserializer: &mut S) -> Result<Self, Error> {
        let mut result = 0;
        let mut shift = 0u32;
        loop {
            let mut chunk = 0;
            deserializer.read_exact(core::slice::from_mut(&mut chunk))?;
            if chunk > 0 && shift + chunk.ilog2() >= usize::BITS {
                return Err(Error::BadFormat);
            }
            result |= ((chunk & 0x7f) as usize) << shift;
            if chunk & 0x80 == 0 {
                break;
            }
            shift = shift.checked_add(7).ok_or(Error::BadFormat)?;
        }
        Ok(result)
    }
}

impl Serialize for [u8] {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), Error> {
        self.len().serialize(serializer)?;
        serializer.write_all(self)
    }
}

impl Deserialize for Vec<u8> {
    fn deserialize<S: Deserializer>(deserializer: &mut S) -> Result<Self, Error> {
        let len = usize::deserialize(deserializer)?;
        let mut data = vec![0; len];
        deserializer.read_exact(&mut data)?;
        Ok(data)
    }
}

impl Serialize for Request {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), Error> {
        match self {
            Request::GenerateKey { key } => {
                serializer.write_all(&[0])?;
                key.serialize(serializer)?;
            }
            Request::DeleteKey { key } => {
                serializer.write_all(&[1])?;
                key.serialize(serializer)?;
            }
            Request::Encrypt { key, nonce, data } => {
                serializer.write_all(&[2])?;
                key.serialize(serializer)?;
                serializer.write_all(nonce)?;
                data.serialize(serializer)?;
            }
            Request::Decrypt { key, nonce, data } => {
                serializer.write_all(&[3])?;
                key.serialize(serializer)?;
                serializer.write_all(nonce)?;
                data.serialize(serializer)?;
            }
            Request::ImportKey { key, secret } => {
                serializer.write_all(&[4])?;
                key.serialize(serializer)?;
                serializer.write_all(secret)?;
            }
            Request::ExportKey { key } => {
                serializer.write_all(&[5])?;
                key.serialize(serializer)?;
            }
        }
        serializer.flush()
    }
}

impl Deserialize for Request {
    fn deserialize<S: Deserializer>(deserializer: &mut S) -> Result<Self, Error> {
        let mut tag = 0;
        deserializer.read_exact(core::slice::from_mut(&mut tag))?;
        Ok(match tag {
            0 => {
                let key = usize::deserialize(deserializer)?;
                Request::GenerateKey { key }
            }
            1 => {
                let key = usize::deserialize(deserializer)?;
                Request::DeleteKey { key }
            }
            2 => {
                let key = usize::deserialize(deserializer)?;
                let mut nonce = [0; 8];
                deserializer.read_exact(&mut nonce)?;
                let data = <Vec<u8>>::deserialize(deserializer)?;
                Request::Encrypt { key, nonce, data }
            }
            3 => {
                let key = usize::deserialize(deserializer)?;
                let mut nonce = [0; 8];
                deserializer.read_exact(&mut nonce)?;
                let data = <Vec<u8>>::deserialize(deserializer)?;
                Request::Decrypt { key, nonce, data }
            }
            4 => {
                let key = usize::deserialize(deserializer)?;
                let mut secret = [0; 16];
                deserializer.read_exact(&mut secret)?;
                Request::ImportKey { key, secret }
            }
            5 => {
                let key = usize::deserialize(deserializer)?;
                Request::ExportKey { key }
            }
            _ => return Err(Error::BadFormat),
        })
    }
}

impl Serialize for Response {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), Error> {
        match self {
            Response::GenerateKey => serializer.write_all(&[0])?,
            Response::DeleteKey => serializer.write_all(&[1])?,
            Response::Encrypt { data } => {
                serializer.write_all(&[2])?;
                data.serialize(serializer)?;
            }
            Response::Decrypt { data } => {
                serializer.write_all(&[3])?;
                data.serialize(serializer)?;
            }
            Response::ImportKey => serializer.write_all(&[4])?,
            Response::ExportKey { secret } => {
                serializer.write_all(&[5])?;
                serializer.write_all(secret)?;
            }
        }
        serializer.flush()
    }
}

impl Deserialize for Response {
    fn deserialize<S: Deserializer>(deserializer: &mut S) -> Result<Self, Error> {
        let mut tag = 0;
        deserializer.read_exact(core::slice::from_mut(&mut tag))?;
        Ok(match tag {
            0 => Response::GenerateKey,
            1 => Response::DeleteKey,
            2 => {
                let data = <Vec<u8>>::deserialize(deserializer)?;
                Response::Encrypt { data }
            }
            3 => {
                let data = <Vec<u8>>::deserialize(deserializer)?;
                Response::Decrypt { data }
            }
            4 => Response::ImportKey,
            5 => {
                let mut secret = [0; 16];
                deserializer.read_exact(&mut secret)?;
                Response::ExportKey { secret }
            }
            _ => return Err(Error::BadFormat),
        })
    }
}

impl Serialize for Error {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), Error> {
        let tag = match self {
            Error::Unknown => 0,
            Error::BadHandle => 1,
            Error::BadFormat => 2,
            Error::CryptoError => 3,
            Error::RngError => 4,
            Error::StoreError => 5,
            Error::UsbError => 6,
        };
        tag.serialize(serializer)
    }
}

impl Deserialize for Error {
    fn deserialize<S: Deserializer>(deserializer: &mut S) -> Result<Self, Error> {
        let tag = usize::deserialize(deserializer)?;
        Ok(match tag {
            0 => Error::Unknown,
            1 => Error::BadHandle,
            2 => Error::BadFormat,
            3 => Error::CryptoError,
            4 => Error::RngError,
            5 => Error::StoreError,
            6 => Error::UsbError,
            _ => return Err(Error::BadFormat),
        })
    }
}

impl<T: Serialize, E: Serialize> Serialize for Result<T, E> {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), Error> {
        match self {
            Ok(x) => {
                serializer.write_all(&[0])?;
                x.serialize(serializer)?;
            }
            Err(e) => {
                serializer.write_all(&[1])?;
                e.serialize(serializer)?;
            }
        }
        Ok(())
    }
}

impl<T: Deserialize, E: Deserialize> Deserialize for Result<T, E> {
    fn deserialize<S: Deserializer>(deserializer: &mut S) -> Result<Self, Error> {
        let tag = usize::deserialize(deserializer)?;
        Ok(match tag {
            0 => Ok(T::deserialize(deserializer)?),
            1 => Err(E::deserialize(deserializer)?),
            _ => return Err(Error::BadFormat),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Serializer for Vec<u8> {
        fn write_all(&mut self, data: &[u8]) -> Result<(), Error> {
            self.extend_from_slice(data);
            Ok(())
        }

        fn flush(&mut self) -> Result<(), Error> {
            Ok(())
        }
    }

    impl<'a> Deserializer for &'a [u8] {
        fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Error> {
            if self.len() < data.len() {
                return Err(Error::UsbError);
            }
            let (head, tail) = self.split_at(data.len());
            data.copy_from_slice(head);
            *self = tail;
            Ok(())
        }
    }

    fn serialize<T: Serialize>(x: &T) -> Result<Vec<u8>, Error> {
        let mut r = Vec::new();
        x.serialize(&mut r)?;
        Ok(r)
    }

    fn deserialize<T: Deserialize>(mut xs: &[u8]) -> Result<T, Error> {
        T::deserialize(&mut xs)
    }

    #[test]
    fn serialize_usize() {
        assert_eq!(serialize(&0x00usize), Ok(vec![0x00]));
        assert_eq!(serialize(&0x01usize), Ok(vec![0x01]));
        assert_eq!(serialize(&0x7fusize), Ok(vec![0x7f]));
        assert_eq!(serialize(&0x80usize), Ok(vec![0x80, 0x01]));
        assert_eq!(
            serialize(&0x01020408102040usize),
            Ok(vec![0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0x40])
        );
        assert_eq!(
            serialize(&0xffff_ffff_ffff_ffffusize),
            Ok(vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01])
        );
    }

    #[test]
    fn deserialize_usize() {
        assert_eq!(deserialize(&[0x00]), Ok(0x00));
        assert_eq!(deserialize(&[0x01]), Ok(0x01));
        assert_eq!(deserialize(&[0x7f]), Ok(0x7f));
        assert_eq!(deserialize(&[0x80, 0x01]), Ok(0x80));
        assert_eq!(deserialize(&[0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0x40]), Ok(0x01020408102040));
        assert_eq!(
            deserialize(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]),
            Ok(0xffff_ffff_ffff_ffffusize)
        );
    }
}
