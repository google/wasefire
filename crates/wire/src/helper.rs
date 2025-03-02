// Copyright 2024 Google LLC
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

use alloc::vec::Vec;

use wasefire_error::{Code, Error};

use crate::reader::Reader;
use crate::writer::Writer;

pub(crate) trait Byte: Copy {
    fn encode(self) -> u8;
    fn decode(x: u8) -> Result<Self, Error>;
}

impl Byte for bool {
    fn encode(self) -> u8 {
        self as u8
    }
    fn decode(x: u8) -> Result<Self, Error> {
        match x {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::user(Code::InvalidArgument)),
        }
    }
}

impl Byte for u8 {
    fn encode(self) -> u8 {
        self
    }
    fn decode(x: u8) -> Result<Self, Error> {
        Ok(x)
    }
}

impl Byte for i8 {
    fn encode(self) -> u8 {
        self as u8
    }
    fn decode(x: u8) -> Result<Self, Error> {
        Ok(x as i8)
    }
}

pub(crate) fn encode_byte<T: Byte>(byte: T, writer: &mut Writer) {
    writer.put_copy(&[byte.encode()]);
}

pub(crate) fn decode_byte<T: Byte>(reader: &mut Reader) -> Result<T, Error> {
    T::decode(reader.get(1)?[0])
}

pub(crate) trait Varint: Copy + Default + Eq {
    const BITS: usize;
    type Buffer: Default + AsMut<[u8]>;
    fn encode(&mut self) -> u8;
    fn decode(&mut self, i: usize, x: u8);
}

pub(crate) trait Zigzag: Copy {
    type Unsigned: Varint;
    fn encode(self) -> Self::Unsigned;
    fn decode(x: Self::Unsigned) -> Self;
}

macro_rules! impl_varint {
    ($u:tt $i:tt) => {
        impl Varint for $u {
            const BITS: usize = $u::BITS as usize;
            type Buffer = [u8; ($u::BITS as usize).div_ceil(7)];
            fn encode(&mut self) -> u8 {
                let x = (*self & 0x7f) as u8;
                *self >>= 7;
                x
            }
            fn decode(&mut self, i: usize, x: u8) {
                *self |= (x as $u) << 7 * i;
            }
        }
        impl Zigzag for $i {
            type Unsigned = $u;
            fn encode(self) -> $u {
                let mask = if self < 0 { $i::MAX } else { 0 };
                (self ^ mask).rotate_left(1) as $u
            }
            fn decode(x: $u) -> $i {
                let mask = if x & 1 != 0 { $i::MAX } else { 0 };
                (x as $i).rotate_right(1) ^ mask
            }
        }
    };
}
impl_varint!(u16 i16);
impl_varint!(u32 i32);
impl_varint!(u64 i64);
impl_varint!(usize isize);

pub(crate) fn encode_varint<T: Varint>(mut value: T, writer: &mut Writer) {
    let mut output = T::Buffer::default();
    let output = output.as_mut();
    let mut len = 0;
    let zero = T::default();
    while len == 0 || value != zero {
        output[len] = 0x80 | value.encode();
        len += 1;
    }
    output[len - 1] &= 0x7f;
    writer.put_copy(&output[.. len]);
}

pub(crate) fn decode_varint<T: Varint>(reader: &mut Reader) -> Result<T, Error> {
    let mut output = T::default();
    let max_len = T::BITS.div_ceil(7);
    let last_mask = u8::MAX << (T::BITS - (max_len - 1) * 7);
    for count in 0 .. max_len {
        let byte = reader.get(1)?[0];
        if count != 0 && byte == 0 {
            return Err(Error::user(Code::InvalidArgument));
        }
        if count == max_len - 1 && byte & last_mask != 0 {
            return Err(Error::user(Code::InvalidArgument));
        }
        output.decode(count, byte & 0x7f);
        if 0x80 & byte == 0 {
            break;
        }
    }
    Ok(output)
}

pub(crate) fn encode_zigzag<T: Zigzag>(value: T, writer: &mut Writer) {
    encode_varint(value.encode(), writer)
}

pub(crate) fn decode_zigzag<T: Zigzag>(reader: &mut Reader) -> Result<T, Error> {
    decode_varint(reader).map(T::decode)
}

pub(crate) fn encode_length(len: usize, writer: &mut Writer) -> Result<(), Error> {
    let len = u32::try_from(len).map_err(|_| Error::user(Code::InvalidLength))?;
    <u32 as crate::internal::Wire>::encode(&len, writer)
}

pub(crate) fn decode_length(reader: &mut Reader) -> Result<usize, Error> {
    Ok(<u32 as crate::internal::Wire>::decode(reader)? as usize)
}

type Encode<'a, T> = fn(&T, &mut Writer<'a>) -> Result<(), Error>;
type Decode<'a, T> = fn(&mut Reader<'a>) -> Result<T, Error>;

pub(crate) fn encode_array_dyn<'a, T>(
    values: &[T], writer: &mut Writer<'a>, element: Encode<'a, T>,
) -> Result<(), Error> {
    values.iter().try_for_each(|x| element(x, writer))
}

pub(crate) fn decode_array_dyn<'a, T>(
    len: usize, reader: &mut Reader<'a>, element: Decode<'a, T>,
) -> Result<Vec<T>, Error> {
    (0 .. len).map(|_| element(reader)).collect()
}

pub(crate) fn encode_array<'a, T, const N: usize>(
    values: &[T; N], writer: &mut Writer<'a>, element: Encode<'a, T>,
) -> Result<(), Error> {
    encode_array_dyn(values, writer, element)
}

pub(crate) fn decode_array<'a, T, const N: usize>(
    reader: &mut Reader<'a>, element: Decode<'a, T>,
) -> Result<[T; N], Error> {
    core::array::try_from_fn(|_| element(reader))
}

pub(crate) fn encode_slice<'a, T>(
    values: &[T], writer: &mut Writer<'a>, element: Encode<'a, T>,
) -> Result<(), Error> {
    encode_length(values.len(), writer)?;
    encode_array_dyn(values, writer, element)
}

pub(crate) fn decode_slice<'a, T>(
    reader: &mut Reader<'a>, element: Decode<'a, T>,
) -> Result<Vec<T>, Error> {
    let len = decode_length(reader)?;
    decode_array_dyn(len, reader, element)
}
