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

//! Wasefire wire format.
//!
//! This crate provides a binary format for a wire used as an RPC from a large host to a small
//! device. The format is compact and canonical, in particular it is not self-describing.
//! Compatibility is encoded with tags of a top-level enum, in particular RPC messages are never
//! changed but instead duplicated to a new variant. The host supports all variants because it is
//! not constrained. The device only supports the latest versions to minimize binary footprint. The
//! host and the device are written in Rust, so wire types are defined in Rust. The data model is
//! simple and contains builtin types, arrays, slices, structs, enums, and supports recursion.
//!
//! Alternatives like serde (with postcard) or protocol buffers solve a more general problem than
//! this use-case. The main differences are:
//!
//! - Not self-describing: the model is simpler and more robust (smaller code footprint on device).
//! - No special cases for options and maps: those are encoded from basic types.
//! - No need for tagged and optional fields: full messages are versioned.
//! - Variant tags can be explicit, and thus feature-gated to reduce device code size.
//! - Wire types are only used to represent wire data, they are not used as regular data types.
//! - Wire types only borrow from the wire, and do so in a covariant way.
//! - Wire types can be inspected programmatically for unit testing.
//! - Users can't implement the wire trait: they can only derive it.

#![no_std]
#![feature(array_try_from_fn)]
#![feature(doc_auto_cfg)]
#![feature(never_type)]
#![feature(try_blocks)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::convert::Infallible;
use core::mem::{ManuallyDrop, MaybeUninit};

use wasefire_error::{Code, Error};
pub use wasefire_wire_derive::Wire;
use wasefire_wire_derive::internal_wire;

#[cfg(feature = "schema")]
use crate::internal::{Builtin, Rules};
use crate::reader::Reader;
use crate::writer::Writer;

mod helper;
pub mod internal;
mod reader;
#[cfg(feature = "schema")]
pub mod schema;
mod writer;

pub trait Wire<'a>: internal::Wire<'a> {}
impl<'a, T: internal::Wire<'a>> Wire<'a> for T {}

pub fn encode_suffix<'a, T: Wire<'a>>(data: &mut Vec<u8>, value: &T) -> Result<(), Error> {
    let mut writer = Writer::new();
    value.encode(&mut writer)?;
    Ok(writer.finalize(data))
}

pub fn encode<'a, T: Wire<'a>>(value: &T) -> Result<Box<[u8]>, Error> {
    let mut data = Vec::new();
    encode_suffix(&mut data, value)?;
    Ok(data.into_boxed_slice())
}

pub fn decode_prefix<'a, T: Wire<'a>>(data: &mut &'a [u8]) -> Result<T, Error> {
    let mut reader = Reader::new(data);
    let value = T::decode(&mut reader)?;
    *data = reader.finalize();
    Ok(value)
}

pub fn decode<'a, T: Wire<'a>>(mut data: &'a [u8]) -> Result<T, Error> {
    let value = decode_prefix(&mut data)?;
    Error::user(Code::InvalidLength).check(data.is_empty())?;
    Ok(value)
}

pub struct Yoke<T: Wire<'static>> {
    // TODO(https://github.com/rust-lang/rust/issues/118166): Use MaybeDangling.
    value: MaybeUninit<T>,
    data: *mut [u8],
}

impl<T: Wire<'static>> core::fmt::Debug for Yoke<T>
where for<'a> T::Type<'a>: core::fmt::Debug
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <T::Type<'_> as core::fmt::Debug>::fmt(self.get(), f)
    }
}

impl<T: Wire<'static>> Drop for Yoke<T> {
    fn drop(&mut self) {
        // SAFETY: data comes from into_raw and has been used linearly since then.
        drop(unsafe { Box::from_raw(self.data) });
    }
}

impl<T: Wire<'static>> Yoke<T> {
    fn take(self) -> (T, *mut [u8]) {
        let this = ManuallyDrop::new(self);
        (unsafe { this.value.assume_init_read() }, this.data)
    }
}

impl<T: Wire<'static>> Yoke<T> {
    pub fn get(&self) -> &<T as internal::Wire<'static>>::Type<'_> {
        // SAFETY: We only read from value which borrows from data.
        unsafe { core::mem::transmute(&self.value) }
    }

    pub fn map<S: Wire<'static>, F: for<'a> FnOnce(T) -> S>(self, f: F) -> Yoke<S> {
        let (value, data) = self.take();
        Yoke { value: MaybeUninit::new(f(value)), data }
    }

    pub fn try_map<S: Wire<'static>, E, F: for<'a> FnOnce(T) -> Result<S, E>>(
        self, f: F,
    ) -> Result<Yoke<S>, E> {
        let (value, data) = self.take();
        Ok(Yoke { value: MaybeUninit::new(f(value)?), data })
    }
}

pub fn decode_yoke<T: Wire<'static>>(data: Box<[u8]>) -> Result<Yoke<T>, Error> {
    let data = Box::into_raw(data);
    // SAFETY: decode does not leak its input in other ways than in its result.
    let value = MaybeUninit::new(decode::<T>(unsafe { &*data })?);
    Ok(Yoke { value, data })
}

macro_rules! impl_builtin {
    ($t:tt $T:tt $encode:tt $decode:tt) => {
        impl<'a> internal::Wire<'a> for $t {
            type Type<'b> = $t;
            #[cfg(feature = "schema")]
            fn schema(rules: &mut Rules) {
                if rules.builtin::<Self::Type<'static>>(Builtin::$T) {}
            }
            fn encode(&self, writer: &mut Writer<'a>) -> Result<(), Error> {
                Ok(helper::$encode(*self, writer))
            }
            fn decode(reader: &mut Reader<'a>) -> Result<Self, Error> {
                helper::$decode(reader)
            }
        }
    };
}
impl_builtin!(bool Bool encode_byte decode_byte);
impl_builtin!(u8 U8 encode_byte decode_byte);
impl_builtin!(i8 I8 encode_byte decode_byte);
impl_builtin!(u16 U16 encode_varint decode_varint);
impl_builtin!(i16 I16 encode_zigzag decode_zigzag);
impl_builtin!(u32 U32 encode_varint decode_varint);
impl_builtin!(i32 I32 encode_zigzag decode_zigzag);
impl_builtin!(u64 U64 encode_varint decode_varint);
impl_builtin!(i64 I64 encode_zigzag decode_zigzag);
impl_builtin!(usize Usize encode_varint decode_varint);
impl_builtin!(isize Isize encode_zigzag decode_zigzag);

impl<'a> internal::Wire<'a> for &'a str {
    type Type<'b> = &'b str;
    #[cfg(feature = "schema")]
    fn schema(rules: &mut Rules) {
        if rules.builtin::<Self::Type<'static>>(Builtin::Str) {}
    }
    fn encode(&self, writer: &mut Writer<'a>) -> Result<(), Error> {
        helper::encode_length(self.len(), writer)?;
        writer.put_share(self.as_bytes());
        Ok(())
    }
    fn decode(reader: &mut Reader<'a>) -> Result<Self, Error> {
        let len = helper::decode_length(reader)?;
        core::str::from_utf8(reader.get(len)?).map_err(|_| Error::user(Code::InvalidArgument))
    }
}

impl<'a> internal::Wire<'a> for () {
    type Type<'b> = ();
    #[cfg(feature = "schema")]
    fn schema(rules: &mut Rules) {
        if rules.struct_::<Self::Type<'static>>(Vec::new()) {}
    }
    fn encode(&self, _writer: &mut Writer<'a>) -> Result<(), Error> {
        Ok(())
    }
    fn decode(_reader: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(())
    }
}

macro_rules! impl_tuple {
    (($($i:tt $t:tt),*), $n:tt) => {
        impl<'a, $($t: Wire<'a>),*> internal::Wire<'a> for ($($t),*) {
            type Type<'b> = ($($t::Type<'b>),*);
            #[cfg(feature = "schema")]
            fn schema(rules: &mut Rules) {
                let mut fields = Vec::with_capacity($n);
                $(fields.push((None, internal::type_id::<$t>()));)*
                if rules.struct_::<Self::Type<'static>>(fields) {
                    $(<$t>::schema(rules);)*
                }
            }
            fn encode(&self, writer: &mut Writer<'a>) -> Result<(), Error> {
                $(self.$i.encode(writer)?;)*
                Ok(())
            }
            fn decode(reader: &mut Reader<'a>) -> Result<Self, Error> {
                Ok(($(<$t>::decode(reader)?),*))
            }
        }
    };
}
impl_tuple!((0 T, 1 S), 2);
impl_tuple!((0 T, 1 S, 2 R), 3);
impl_tuple!((0 T, 1 S, 2 R, 3 Q), 4);
impl_tuple!((0 T, 1 S, 2 R, 3 Q, 4 P), 5);

impl<'a, const N: usize> internal::Wire<'a> for &'a [u8; N] {
    type Type<'b> = &'b [u8; N];
    #[cfg(feature = "schema")]
    fn schema(rules: &mut Rules) {
        if rules.array::<Self::Type<'static>, u8>(N) {
            internal::schema::<u8>(rules);
        }
    }
    fn encode(&self, writer: &mut Writer<'a>) -> Result<(), Error> {
        Ok(writer.put_share(*self))
    }
    fn decode(reader: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(reader.get(N)?.try_into().unwrap())
    }
}

impl<'a> internal::Wire<'a> for &'a [u8] {
    type Type<'b> = &'b [u8];
    #[cfg(feature = "schema")]
    fn schema(rules: &mut Rules) {
        if rules.slice::<Self::Type<'static>, u8>() {
            internal::schema::<u8>(rules);
        }
    }
    fn encode(&self, writer: &mut Writer<'a>) -> Result<(), Error> {
        helper::encode_length(self.len(), writer)?;
        writer.put_share(self);
        Ok(())
    }
    fn decode(reader: &mut Reader<'a>) -> Result<Self, Error> {
        let len = helper::decode_length(reader)?;
        reader.get(len)
    }
}

impl<'a, T: Wire<'a>, const N: usize> internal::Wire<'a> for [T; N] {
    type Type<'b> = [T::Type<'b>; N];
    #[cfg(feature = "schema")]
    fn schema(rules: &mut Rules) {
        if rules.array::<Self::Type<'static>, T::Type<'static>>(N) {
            internal::schema::<T>(rules);
        }
    }
    fn encode(&self, writer: &mut Writer<'a>) -> Result<(), Error> {
        helper::encode_array(self, writer, T::encode)
    }
    fn decode(reader: &mut Reader<'a>) -> Result<Self, Error> {
        helper::decode_array(reader, T::decode)
    }
}

impl<'a, T: Wire<'a>> internal::Wire<'a> for Vec<T> {
    type Type<'b> = Vec<T::Type<'b>>;
    #[cfg(feature = "schema")]
    fn schema(rules: &mut Rules) {
        if rules.slice::<Self::Type<'static>, T::Type<'static>>() {
            internal::schema::<T>(rules);
        }
    }
    fn encode(&self, writer: &mut Writer<'a>) -> Result<(), Error> {
        helper::encode_slice(self, writer, T::encode)
    }
    fn decode(reader: &mut Reader<'a>) -> Result<Self, Error> {
        helper::decode_slice(reader, T::decode)
    }
}

impl<'a, T: Wire<'a>> internal::Wire<'a> for Box<T> {
    type Type<'b> = Box<T::Type<'b>>;
    #[cfg(feature = "schema")]
    fn schema(rules: &mut Rules) {
        let mut fields = Vec::with_capacity(1);
        fields.push((None, internal::type_id::<T>()));
        if rules.struct_::<Self::Type<'static>>(fields) {
            internal::schema::<T>(rules);
        }
    }
    fn encode(&self, writer: &mut Writer<'a>) -> Result<(), Error> {
        T::encode(self, writer)
    }
    fn decode(reader: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(Box::new(T::decode(reader)?))
    }
}

impl<'a> internal::Wire<'a> for Error {
    type Type<'b> = Error;
    #[cfg(feature = "schema")]
    fn schema(rules: &mut Rules) {
        let mut fields = Vec::with_capacity(2);
        fields.push((Some("space"), internal::type_id::<u8>()));
        fields.push((Some("code"), internal::type_id::<u16>()));
        if rules.struct_::<Self::Type<'static>>(fields) {
            internal::schema::<u8>(rules);
            internal::schema::<u16>(rules);
        }
    }
    fn encode(&self, writer: &mut Writer<'a>) -> Result<(), Error> {
        self.space().encode(writer)?;
        self.code().encode(writer)
    }
    fn decode(reader: &mut Reader<'a>) -> Result<Self, Error> {
        let space = u8::decode(reader)?;
        let code = u16::decode(reader)?;
        Ok(Error::new(space, code))
    }
}

impl<'a> internal::Wire<'a> for ! {
    type Type<'b> = !;
    #[cfg(feature = "schema")]
    fn schema(rules: &mut Rules) {
        if rules.enum_::<Self::Type<'static>>(Vec::new()) {}
    }
    fn encode(&self, _: &mut Writer<'a>) -> Result<(), Error> {
        match *self {}
    }
    fn decode(_: &mut Reader<'a>) -> Result<Self, Error> {
        Err(Error::user(Code::InvalidArgument))
    }
}

#[internal_wire]
#[wire(crate = crate)]
enum Infallible {}

#[internal_wire]
#[wire(crate = crate, where = T: Wire<'wire>)]
enum Option<T> {
    None,
    Some(T),
}

#[internal_wire]
#[wire(crate = crate, where = T: Wire<'wire>, where = E: Wire<'wire>)]
enum Result<T, E> {
    Ok(T),
    Err(E),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::View;

    #[test]
    fn encode_varint() {
        #[track_caller]
        fn test<T: Wire<'static>>(value: T, expected: &[u8]) {
            assert_eq!(&encode(&value).unwrap()[..], expected);
        }
        test::<u16>(0x00, &[0x00]);
        test::<u16>(0x01, &[0x01]);
        test::<u16>(0x7f, &[0x7f]);
        test::<u16>(0x80, &[0x80, 0x01]);
        test::<u16>(0xff, &[0xff, 0x01]);
        test::<u16>(0xfffe, &[0xfe, 0xff, 0x03]);
        test::<i16>(0, &[0x00]);
        test::<i16>(-1, &[0x01]);
        test::<i16>(1, &[0x02]);
        test::<i16>(-2, &[0x03]);
        test::<i16>(i16::MAX, &[0xfe, 0xff, 0x03]);
        test::<i16>(i16::MIN, &[0xff, 0xff, 0x03]);
    }

    #[test]
    fn decode_varint() {
        #[track_caller]
        fn test<T: Wire<'static> + Eq + std::fmt::Debug>(data: &'static [u8], expected: Option<T>) {
            assert_eq!(decode(data).ok(), expected);
        }
        test::<u16>(&[0x00], Some(0x00));
        test::<u16>(&[0x01], Some(0x01));
        test::<u16>(&[0x7f], Some(0x7f));
        test::<u16>(&[0x80, 0x01], Some(0x80));
        test::<u16>(&[0xff, 0x01], Some(0xff));
        test::<u16>(&[0xfe, 0xff, 0x03], Some(0xfffe));
        test::<u16>(&[0xfe, 0x00], None);
        test::<u16>(&[0xfe, 0xff, 0x00], None);
        test::<u16>(&[0xfe, 0xff, 0x04], None);
        test::<u16>(&[0xfe, 0xff, 0x40], None);
        test::<u16>(&[0xfe, 0xff, 0x80], None);
        test::<u16>(&[0xfe, 0xff, 0x80, 0x01], None);
        test::<i16>(&[0x00], Some(0));
        test::<i16>(&[0x01], Some(-1));
        test::<i16>(&[0x02], Some(1));
        test::<i16>(&[0x03], Some(-2));
        test::<i16>(&[0xfe, 0xff, 0x03], Some(i16::MAX));
        test::<i16>(&[0xff, 0xff, 0x03], Some(i16::MIN));
    }

    #[track_caller]
    fn assert_schema<'a, T: Wire<'a>>(expected: &str) {
        let x = View::new::<T>();
        assert_eq!(std::format!("{x}"), expected);
    }

    #[test]
    fn display_schema() {
        assert_schema::<bool>("bool");
        assert_schema::<u8>("u8");
        assert_schema::<&str>("str");
        assert_schema::<Result<&str, &[u8]>>("{Ok=0:str Err=1:[u8]}");
        assert_schema::<Option<[u8; 42]>>("{None=0:() Some=1:[u8; 42]}");
    }

    #[test]
    fn derive_struct() {
        #[derive(Wire)]
        #[wire(crate = crate)]
        struct Foo1 {
            bar: u8,
            baz: u32,
        }
        assert_schema::<Foo1>("(bar:u8 baz:u32)");

        #[derive(Wire)]
        #[wire(crate = crate)]
        struct Foo2<'a> {
            bar: &'a str,
            baz: Option<&'a [u8]>,
        }
        assert_schema::<Foo2>("(bar:str baz:{None=0:() Some=1:[u8]})");
    }

    #[test]
    fn derive_enum() {
        #[derive(Wire)]
        #[wire(crate = crate)]
        enum Foo1 {
            Bar,
            Baz(u32),
        }
        assert_schema::<Foo1>("{Bar=0:() Baz=1:u32}");

        #[derive(Wire)]
        #[wire(crate = crate)]
        enum Foo2<'a> {
            #[wire(tag = 1)]
            Bar(&'a str),
            #[wire(tag = 0)]
            Baz((), Option<&'a [u8]>),
        }
        assert_schema::<Foo2>("{Bar=1:str Baz=0:{None=0:() Some=1:[u8]}}");
    }

    #[test]
    fn recursive_view() {
        #[derive(Debug, Wire, PartialEq, Eq)]
        #[wire(crate = crate)]
        enum List {
            Nil,
            Cons(u8, Box<List>),
        }
        assert_schema::<List>("<1>:{Nil=0:() Cons=1:(u8 <1>)}");
        let value = List::Cons(13, Box::new(List::Cons(42, Box::new(List::Nil))));
        let data = encode(&value).unwrap();
        let view = View::new::<List>();
        assert!(view.validate(&data).is_ok());
        assert_eq!(decode::<List>(&data).unwrap(), value);
    }

    #[test]
    fn yoke() {
        type T = Result<&'static [u8], ()>;
        let value: T = Ok(b"hello");
        let data = encode(&value).unwrap();
        let yoke = decode_yoke::<T>(data).unwrap();
        let bytes = yoke.try_map(|x| x).unwrap();
        assert_eq!(bytes.get(), b"hello");
    }
}
