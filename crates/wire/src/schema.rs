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

//! Provides a schema of the wire format.
//!
//! This is only meant to be used during testing, to ensure a wire format is monotonic.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::any::TypeId;
use core::fmt::Display;
use std::collections::HashMap;
use std::sync::Mutex;

use wasefire_error::{Code, Error};

use crate::internal::{Builtin, Rule, RuleEnum, RuleStruct, Rules, Wire};
use crate::reader::Reader;
use crate::{helper, internal};

#[derive(Debug, Clone, PartialEq, Eq, wasefire_wire_derive::Wire)]
#[wire(crate = crate)]
pub enum View<'a> {
    #[wire(tag = 0)]
    Builtin(Builtin),
    #[wire(tag = 1)]
    Array(Box<View<'a>>, usize),
    #[wire(tag = 2)]
    Slice(Box<View<'a>>),
    #[wire(tag = 3)]
    Struct(ViewStruct<'a>),
    #[wire(tag = 4)]
    Enum(ViewEnum<'a>),
    #[wire(tag = 5)]
    RecUse(usize),
    #[wire(tag = 6)]
    RecNew(usize, Box<View<'a>>),
}

pub type ViewStruct<'a> = Vec<(Option<&'a str>, View<'a>)>;
pub type ViewEnum<'a> = Vec<(&'a str, u32, ViewStruct<'a>)>;

impl View<'static> {
    pub fn new<'a, T: Wire<'a>>() -> View<'static> {
        let mut rules = Rules::default();
        T::schema(&mut rules);
        Traverse::new(&rules).extract_or_empty(TypeId::of::<T::Static>())
    }
}

struct Traverse<'a> {
    rules: &'a Rules,
    next: usize,
    path: Vec<(TypeId, Option<usize>)>,
}

impl<'a> Traverse<'a> {
    fn new(rules: &'a Rules) -> Self {
        Traverse { rules, next: 0, path: Vec::new() }
    }

    fn extract_or_empty(&mut self, id: TypeId) -> View<'static> {
        match self.extract(id) {
            Some(x) => x,
            None => View::Enum(Vec::new()),
        }
    }

    fn extract(&mut self, id: TypeId) -> Option<View<'static>> {
        if let Some((_, rec)) = self.path.iter_mut().find(|(x, _)| *x == id) {
            let rec = rec.get_or_insert_with(|| {
                self.next += 1;
                self.next
            });
            return Some(View::RecUse(*rec));
        }
        self.path.push((id, None));
        let result: Option<_> = try {
            match self.rules.get(id) {
                Rule::Builtin(x) => View::Builtin(*x),
                Rule::Array(_, 0) => View::Struct(Vec::new()),
                Rule::Array(x, 1) => self.extract(*x)?,
                Rule::Array(x, n) => View::Array(Box::new(self.extract(*x)?), *n),
                Rule::Slice(x) => View::Slice(Box::new(self.extract_or_empty(*x))),
                Rule::Struct(xs) => match xs[..] {
                    [(None, x)] => self.extract(x)?,
                    _ => View::Struct(self.extract_struct(xs)?),
                },
                Rule::Enum(xs) => View::Enum(self.extract_enum(xs)),
            }
        };
        let (id_, rec) = self.path.pop().unwrap();
        assert_eq!(id_, id);
        let result = result?;
        Some(match rec {
            Some(rec) => View::RecNew(rec, Box::new(result)),
            None => result,
        })
    }

    fn extract_struct(&mut self, xs: &RuleStruct) -> Option<ViewStruct<'static>> {
        xs.iter().map(|(n, x)| Some((*n, self.extract(*x)?))).collect()
    }

    fn extract_enum(&mut self, xs: &RuleEnum) -> ViewEnum<'static> {
        xs.iter().filter_map(|(n, i, xs)| Some((*n, *i, self.extract_struct(xs)?))).collect()
    }
}

impl core::fmt::Display for Builtin {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Builtin::Bool => write!(f, "bool"),
            Builtin::U8 => write!(f, "u8"),
            Builtin::I8 => write!(f, "i8"),
            Builtin::U16 => write!(f, "u16"),
            Builtin::I16 => write!(f, "i16"),
            Builtin::U32 => write!(f, "u32"),
            Builtin::I32 => write!(f, "i32"),
            Builtin::U64 => write!(f, "u64"),
            Builtin::I64 => write!(f, "i64"),
            Builtin::Usize => write!(f, "usize"),
            Builtin::Isize => write!(f, "isize"),
            Builtin::Str => write!(f, "str"),
        }
    }
}

impl<'a> core::fmt::Display for View<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            View::Builtin(x) => write!(f, "{x}"),
            View::Array(x, n) => write!(f, "[{x}; {n}]"),
            View::Slice(x) => write!(f, "[{x}]"),
            View::Struct(xs) => write_fields(f, xs),
            View::Enum(xs) => write_list(f, xs),
            View::RecUse(n) => write!(f, "<{n}>"),
            View::RecNew(n, x) => write!(f, "<{n}>:{x}"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ViewFields<'a>(pub &'a ViewStruct<'a>);

impl<'a> core::fmt::Display for ViewFields<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write_fields(f, self.0)
    }
}

trait List {
    const BEG: char;
    const END: char;
    fn fmt_name(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result;
    fn fmt_item(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result;
}

impl<'a> List for (Option<&'a str>, View<'a>) {
    const BEG: char = '(';
    const END: char = ')';
    fn fmt_name(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self.0 {
            Some(x) => write!(f, "{x}:"),
            None => Ok(()),
        }
    }
    fn fmt_item(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.1.fmt(f)
    }
}

impl<'a> List for (&'a str, u32, ViewStruct<'a>) {
    const BEG: char = '{';
    const END: char = '}';
    fn fmt_name(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}={}:", self.0, self.1)
    }
    fn fmt_item(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write_fields(f, &self.2)
    }
}

fn write_fields(f: &mut core::fmt::Formatter, xs: &[(Option<&str>, View)]) -> core::fmt::Result {
    if xs.len() == 1 && xs[0].0.is_none() {
        xs[0].1.fmt(f)
    } else {
        write_list(f, xs)
    }
}

fn write_list<T: List>(f: &mut core::fmt::Formatter, xs: &[T]) -> core::fmt::Result {
    write!(f, "{}", T::BEG)?;
    let mut first = true;
    for x in xs.iter() {
        if !first {
            write!(f, " ")?;
        }
        first = false;
        x.fmt_name(f)?;
        x.fmt_item(f)?;
    }
    write!(f, "{}", T::END)
}

impl<'a> View<'a> {
    /// Returns whether two views are compatible.
    ///
    /// This function is sound but not necessarily complete. If it returns true, then the views are
    /// compatible. But it may also return false for compatible views.
    pub fn compatible(&self, other: &View<'a>) -> bool {
        match (self, other) {
            (View::Builtin(x), View::Builtin(y)) => x == y,
            (View::Array(x, n), View::Array(y, m)) => x == y && n == m,
            (View::Slice(x), View::Slice(y)) => x == y,
            (View::Struct(xs), View::Struct(ys)) => ViewFields(xs).compatible(ViewFields(ys)),
            (View::Enum(xs), View::Enum(ys)) => {
                if xs.len() != ys.len() {
                    return false;
                }
                let xs: HashMap<u32, &ViewStruct> = xs.iter().map(|(_, t, x)| (*t, x)).collect();
                for (_, t, ys) in ys.iter() {
                    match xs.get(t) {
                        Some(xs) if ViewFields(xs).compatible(ViewFields(ys)) => (),
                        _ => return false,
                    }
                }
                true
            }
            (View::RecUse(n), View::RecUse(m)) => n == m,
            (View::RecNew(n, x), View::RecNew(m, y)) => n == m && x == y,
            _ => false,
        }
    }
}

impl<'a> ViewFields<'a> {
    pub fn compatible(self, other: ViewFields<'a>) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        for ((_, x), (_, y)) in self.0.iter().zip(other.0.iter()) {
            if !x.compatible(y) {
                return false;
            }
        }
        true
    }
}

impl<'a> View<'a> {
    /// Validates that serialized data matches the view.
    ///
    /// This function requires a global lock.
    ///
    /// # Panics
    ///
    /// Panics if used concurrently.
    pub fn validate(&self, data: &[u8]) -> Result<(), Error> {
        let _lock = ViewFrame::new(self);
        let _ = crate::decode::<ViewDecoder>(data)?;
        Ok(())
    }

    fn decode(&self, reader: &mut Reader) -> Result<(), Error> {
        match self {
            View::Builtin(Builtin::Bool) => drop(bool::decode(reader)?),
            View::Builtin(Builtin::U8) => drop(u8::decode(reader)?),
            View::Builtin(Builtin::I8) => drop(i8::decode(reader)?),
            View::Builtin(Builtin::U16) => drop(u16::decode(reader)?),
            View::Builtin(Builtin::I16) => drop(i16::decode(reader)?),
            View::Builtin(Builtin::U32) => drop(u32::decode(reader)?),
            View::Builtin(Builtin::I32) => drop(i32::decode(reader)?),
            View::Builtin(Builtin::U64) => drop(u64::decode(reader)?),
            View::Builtin(Builtin::I64) => drop(i64::decode(reader)?),
            View::Builtin(Builtin::Usize) => drop(usize::decode(reader)?),
            View::Builtin(Builtin::Isize) => drop(isize::decode(reader)?),
            View::Builtin(Builtin::Str) => drop(<&str>::decode(reader)?),
            View::Array(x, n) => {
                let _lock = ViewFrame::new(x);
                let _ = helper::decode_array_dyn(*n, reader, decode_view)?;
            }
            View::Slice(x) => {
                let _lock = ViewFrame::new(x);
                let _ = helper::decode_slice(reader, decode_view)?;
            }
            View::Struct(xs) => {
                for (_, x) in xs {
                    x.decode(reader)?;
                }
            }
            View::Enum(xs) => {
                let tag = internal::decode_tag(reader)?;
                let mut found = false;
                for (_, i, xs) in xs {
                    if tag == *i {
                        assert!(!std::mem::replace(&mut found, true));
                        for (_, x) in xs {
                            x.decode(reader)?;
                        }
                    }
                }
                if !found {
                    return Err(Error::user(Code::InvalidArgument));
                }
            }
            View::RecUse(_) => todo!(),
            View::RecNew(_, _) => todo!(),
        }
        Ok(())
    }
}

static VIEW_STACK: Mutex<Vec<&'static View<'static>>> = Mutex::new(Vec::new());

struct ViewFrame(&'static View<'static>);

impl ViewFrame {
    fn new(view: &View) -> Self {
        // TODO(https://github.com/rust-lang/rust-clippy/issues/12860): Remove when fixed.
        #[allow(clippy::unnecessary_cast)]
        // SAFETY: This function is only called when drop is called before the lifetime ends.
        let view = unsafe { &*(view as *const _ as *const View<'static>) };
        VIEW_STACK.lock().unwrap().push(view);
        ViewFrame(view)
    }
}

impl Drop for ViewFrame {
    fn drop(&mut self) {
        assert_eq!(VIEW_STACK.lock().unwrap().pop().unwrap() as *const _, self.0 as *const _);
    }
}

struct ViewDecoder;

impl<'a> internal::Wire<'a> for ViewDecoder {
    type Static = ViewDecoder;
    fn schema(_rules: &mut Rules) {
        unreachable!()
    }
    fn encode(&self, _: &mut internal::Writer<'a>) -> internal::Result<()> {
        unreachable!()
    }
    fn decode(reader: &mut Reader<'a>) -> internal::Result<Self> {
        decode_view(reader)?;
        Ok(ViewDecoder)
    }
}

fn decode_view(reader: &mut Reader) -> Result<(), Error> {
    let view = *VIEW_STACK.lock().unwrap().last().unwrap();
    view.decode(reader)
}
