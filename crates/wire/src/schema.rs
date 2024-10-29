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
use alloc::vec;
use alloc::vec::Vec;
use core::any::TypeId;
use core::fmt::Display;
use std::sync::Mutex;

use wasefire_error::{Code, Error};

use crate::internal::{Builtin, Rule, RuleEnum, RuleStruct, Rules, Wire};
use crate::reader::Reader;
use crate::{helper, internal};

#[derive(Debug, Clone, PartialEq, Eq, wasefire_wire_derive::Wire)]
#[wire(crate = crate)]
pub enum View<'a> {
    Builtin(Builtin),
    Array(Box<View<'a>>, usize),
    Slice(Box<View<'a>>),
    Struct(ViewStruct<'a>),
    Enum(ViewEnum<'a>),
    RecUse(usize),
    RecNew(usize, Box<View<'a>>),
}

pub type ViewStruct<'a> = Vec<(Option<&'a str>, View<'a>)>;
pub type ViewEnum<'a> = Vec<(&'a str, u32, ViewStruct<'a>)>;

impl View<'static> {
    pub fn new<'a, T: Wire<'a>>() -> View<'static> {
        let mut rules = Rules::default();
        T::schema(&mut rules);
        Traverse::new(&rules).extract_or_empty(TypeId::of::<T::Type<'static>>())
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
                Rule::Array(x, n) => View::Array(Box::new(self.extract(*x)?), *n),
                Rule::Slice(x) => View::Slice(Box::new(self.extract_or_empty(*x))),
                Rule::Struct(xs) => View::Struct(self.extract_struct(xs)?),
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
        xs.iter()
            .map(|(n, x)| Some((*n, self.extract(*x)?)))
            .filter(|x| !matches!(x, Some((None, View::Struct(xs))) if xs.is_empty()))
            .collect()
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

impl core::fmt::Display for View<'_> {
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

impl core::fmt::Display for ViewFields<'_> {
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
        match self.0 {
            "" => write!(f, "{}:", self.1),
            _ => write!(f, "{}={}:", self.0, self.1),
        }
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

impl View<'_> {
    /// Simplifies a view preserving wire compatibility.
    ///
    /// Performs the following simplifications:
    /// - Remove field names and use empty names for variants
    /// - `[x; 0]` becomes `()`
    /// - `[x; 1]` becomes `x`
    /// - `[[x; n]; m]` becomes `[x; n * m]`
    /// - `[{}; 2+]` becomes `{}`
    /// - `(xs.. (ys..) zs..)` becomes `(xs.. ys.. zs..)`
    /// - `(xs.. {} zs..)` becomes `{}`
    /// - `(xs.. ys.. zs..)` becomes `(xs.. [y; n] zs..)` if `ys..` is `n` times `y`
    /// - `(x)` becomes `x`
    /// - `{xs.. =t:{} zs..}` becomes `{xs.. zs..}`
    /// - `{xs..}` becomes `{ys..}` where `ys..` is `xs..` sorted by tags
    /// - `<n>:x` (resp `<n>`) becomes `<k>:x` (resp. `<k>`) with `k` the number of recursion
    ///   binders from the root
    pub fn simplify(&self) -> View<'static> {
        self.simplify_(RecStack::Root)
    }

    pub fn simplify_struct(xs: &ViewStruct) -> View<'static> {
        View::simplify_struct_(xs, RecStack::Root)
    }

    fn simplify_(&self, rec: RecStack) -> View<'static> {
        match self {
            View::Builtin(x) => View::Builtin(*x),
            View::Array(_, 0) => View::Struct(Vec::new()),
            View::Array(x, 1) => x.simplify_(rec),
            View::Array(x, n) => match x.simplify_(rec) {
                View::Array(x, m) => View::Array(x, n * m),
                View::Enum(xs) if xs.is_empty() => View::Enum(xs),
                x => View::Array(Box::new(x), *n),
            },
            View::Slice(x) => View::Slice(Box::new(x.simplify_(rec))),
            View::Struct(xs) => View::simplify_struct_(xs, rec),
            View::Enum(xs) => {
                let mut ys = Vec::new();
                for (_, t, xs) in xs {
                    let xs = match View::simplify_struct_(xs, rec) {
                        View::Struct(xs) => xs,
                        View::Enum(xs) if xs.is_empty() => continue,
                        x => vec![(None, x)],
                    };
                    ys.push(("", *t, xs));
                }
                ys.sort_by_key(|(_, t, _)| *t);
                View::Enum(ys)
            }
            View::RecUse(n) => View::RecUse(rec.use_(*n)),
            View::RecNew(n, x) => View::RecNew(rec.len(), Box::new(x.simplify_(rec.new(*n)))),
        }
    }

    fn simplify_struct_(xs: &ViewStruct, rec: RecStack) -> View<'static> {
        let mut ys = Vec::new();
        for (_, x) in xs {
            match x.simplify_(rec) {
                View::Struct(mut xs) => ys.append(&mut xs),
                View::Enum(xs) if xs.is_empty() => return View::Enum(xs),
                y => ys.push((None, y)),
            }
        }
        let mut zs = Vec::new();
        for (_, y) in ys {
            let z = match zs.last_mut() {
                Some((_, z)) => z,
                None => {
                    zs.push((None, y));
                    continue;
                }
            };
            match (z, y) {
                (View::Array(x, n), View::Array(y, m)) if *x == y => *n += m,
                (View::Array(x, n), y) if **x == y => *n += 1,
                (x, View::Array(y, m)) if *x == *y => *x = View::Array(y, m + 1),
                (x, y) if *x == y => *x = View::Array(Box::new(y), 2),
                (_, y) => zs.push((None, y)),
            }
        }
        match zs.len() {
            1 => zs.pop().unwrap().1,
            _ => View::Struct(zs),
        }
    }

    /// Validates that serialized data matches the view.
    ///
    /// This function requires a global lock.
    pub fn validate(&self, data: &[u8]) -> Result<(), Error> {
        static GLOBAL_LOCK: Mutex<()> = Mutex::new(());
        let _global_lock = GLOBAL_LOCK.lock().unwrap();
        let _lock = ViewFrameLock::new(None, self);
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
                let _lock = ViewFrameLock::new(None, x);
                let _ = helper::decode_array_dyn(*n, reader, decode_view)?;
            }
            View::Slice(x) => {
                let _lock = ViewFrameLock::new(None, x);
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
            View::RecUse(rec) => {
                let view = VIEW_STACK
                    .lock()
                    .unwrap()
                    .iter()
                    .find(|x| x.rec == Some(*rec))
                    .ok_or(Error::user(Code::InvalidArgument))?
                    .view;
                view.decode(reader)?;
            }
            View::RecNew(rec, x) => {
                let _lock = ViewFrameLock::new(Some(*rec), x);
                x.decode(reader)?;
            }
        }
        Ok(())
    }
}

static VIEW_STACK: Mutex<Vec<ViewFrame>> = Mutex::new(Vec::new());

#[derive(Copy, Clone)]
struct ViewFrame {
    rec: Option<usize>,
    view: &'static View<'static>,
}

struct ViewFrameLock(ViewFrame);

impl ViewFrame {
    fn key(self) -> (Option<usize>, *const View<'static>) {
        (self.rec, self.view as *const _)
    }
}

impl ViewFrameLock {
    fn new(rec: Option<usize>, view: &View) -> Self {
        // TODO(https://github.com/rust-lang/rust-clippy/issues/12860): Remove when fixed.
        #[expect(clippy::unnecessary_cast)]
        // SAFETY: This function is only called when drop is called before the lifetime ends.
        let view = unsafe { &*(view as *const _ as *const View<'static>) };
        let frame = ViewFrame { rec, view };
        VIEW_STACK.lock().unwrap().push(frame);
        ViewFrameLock(frame)
    }
}

impl Drop for ViewFrameLock {
    fn drop(&mut self) {
        assert_eq!(VIEW_STACK.lock().unwrap().pop().unwrap().key(), self.0.key());
    }
}

struct ViewDecoder;

impl<'a> internal::Wire<'a> for ViewDecoder {
    type Type<'b> = ViewDecoder;
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
    let view = VIEW_STACK.lock().unwrap().last().unwrap().view;
    view.decode(reader)
}

#[derive(Copy, Clone)]
enum RecStack<'a> {
    Root,
    Binder(usize, &'a RecStack<'a>),
}

impl<'a> RecStack<'a> {
    fn use_(&self, x: usize) -> usize {
        match self {
            RecStack::Root => unreachable!(),
            RecStack::Binder(y, r) if x == *y => r.len(),
            RecStack::Binder(_, r) => r.use_(x),
        }
    }

    fn len(&self) -> usize {
        match self {
            RecStack::Root => 0,
            RecStack::Binder(_, r) => 1 + r.len(),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn new(&'a self, x: usize) -> RecStack<'a> {
        RecStack::Binder(x, self)
    }
}
