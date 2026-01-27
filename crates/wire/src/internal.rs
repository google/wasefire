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

//! Internal details exposed for the derive macro.

pub use alloc::vec::Vec;

use wasefire_error::{Code, Error, Space};

#[cfg(feature = "schema")]
pub use self::schema::*;
pub use crate::reader::Reader;
pub use crate::writer::Writer;

pub type Result<T> = core::result::Result<T, Error>;

pub trait Wire<'a>: 'a + Sized {
    type Type<'b>: 'b + Sized + Wire<'b>; // Type<'a> == Self
    #[cfg(feature = "schema")]
    fn schema(rules: &mut Rules);
    fn encode(&self, writer: &mut Writer<'a>) -> Result<()>;
    fn decode(reader: &mut Reader<'a>) -> Result<Self>;
}

pub const INVALID_TAG: Error = Error::new_const(Space::User as u8, Code::InvalidArgument as u16);

pub fn encode_tag(tag: u32, writer: &mut Writer) -> Result<()> {
    <u32 as crate::internal::Wire>::encode(&tag, writer)
}

pub fn decode_tag(reader: &mut Reader) -> Result<u32> {
    <u32 as crate::internal::Wire>::decode(reader)
}

#[cfg(feature = "schema")]
mod schema {
    use alloc::vec::Vec;
    use core::any::TypeId;
    use std::collections::HashMap;
    use std::collections::hash_map::Entry;

    use crate::Wire;

    pub fn schema<'a, T: Wire<'a>>(rules: &mut Rules) {
        T::schema(rules)
    }

    pub fn type_id<'a, T: Wire<'a>>() -> TypeId {
        TypeId::of::<T::Type<'static>>()
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, wasefire_wire_derive::Wire)]
    #[wire(crate = crate, range = 11)]
    pub enum Builtin {
        Bool,
        U8,
        I8,
        U16,
        I16,
        U32,
        I32,
        U64,
        I64,
        Usize,
        Isize,
    }

    #[derive(Debug, Default)]
    pub struct Rules(HashMap<TypeId, Rule>);

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Rule {
        Builtin(Builtin),
        Alias(TypeId),
        Array(TypeId, usize),
        Slice(TypeId),
        Struct(RuleStruct),
        Enum(RuleEnum),
    }
    pub type RuleStruct = Vec<(Option<&'static str>, TypeId)>;
    pub type RuleEnum = Vec<(&'static str, u32, RuleStruct)>;

    impl Rules {
        pub(crate) fn builtin<X: 'static>(&mut self, builtin: Builtin) {
            let _ = self.insert::<X>(Rule::Builtin(builtin));
        }

        pub(crate) fn alias<X: 'static, T: Wire<'static>>(&mut self) {
            if self.insert::<X>(Rule::Alias(TypeId::of::<T>())) {
                T::schema(self);
            }
        }

        pub(crate) fn array<X: 'static, T: Wire<'static>>(&mut self, n: usize) {
            if self.insert::<X>(Rule::Array(TypeId::of::<T>(), n)) {
                T::schema(self);
            }
        }

        pub(crate) fn slice<X: 'static, T: Wire<'static>>(&mut self) {
            if self.insert::<X>(Rule::Slice(TypeId::of::<T>())) {
                T::schema(self);
            }
        }

        pub fn struct_<X: 'static>(&mut self, fields: RuleStruct) -> bool {
            self.insert::<X>(Rule::Struct(fields))
        }

        pub fn enum_<X: 'static>(&mut self, variants: RuleEnum) -> bool {
            self.insert::<X>(Rule::Enum(variants))
        }

        fn insert<T: 'static>(&mut self, rule: Rule) -> bool {
            match self.0.entry(TypeId::of::<T>()) {
                Entry::Occupied(old) => {
                    assert_eq!(old.get(), &rule);
                    false
                }
                Entry::Vacant(x) => {
                    let _ = x.insert(rule);
                    true
                }
            }
        }

        pub(crate) fn get(&self, mut id: TypeId) -> &Rule {
            loop {
                match self.0.get(&id).unwrap() {
                    Rule::Alias(x) => id = *x,
                    x => break x,
                }
            }
        }
    }
}
