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

use alloc::vec::Vec;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::module::*;
use crate::syntax::*;
use crate::*;

// TODO: Introduce untyped values? (to save the tag)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Val {
    I32(u32),
    I64(u64),
    F32(u32),
    F64(u64),
    // TODO: Gate v128 with a feature to save RAM. Or Box it.
    V128(u128),
    Null(RefType),
    Ref(usize),
    RefExtern(usize),
}

#[derive(Debug, Default)]
pub struct Store<'m> {
    singleton: StoreSingleton,
    funcs: Vec<FuncInst<'m>>,
    tables: Vec<TableInst>,
    mems: Vec<MemInst<'m>>,
    globals: Vec<GlobalInst>,
    elems: Vec<ElemInst>,
    datas: Vec<DataInst<'m>>,
    modules: Vec<ModuleInst<'m>>,
}

#[derive(Debug)]
pub struct ModuleInst<'m> {
    pub module: Module<'m>,
    pub funcaddrs: Vec<Addr<FuncInst<'m>>>,
    pub tableaddrs: Vec<Addr<TableInst>>,
    pub memaddrs: Vec<Addr<MemInst<'m>>>,
    pub globaladdrs: Vec<Addr<GlobalInst>>,
    pub exports: Vec<ExportInst<'m>>,
    pub elemaddrs: Vec<usize>,
    pub dataaddrs: Vec<usize>,
}

#[derive(Debug)]
pub enum FuncInst<'m> {
    Guest {
        type_: FuncType<'m>,
        module: Addr<ModuleInst<'m>>,
        code: Parser<'m>,
    },
    Host {
        type_: FuncType<'m>,
        // hostcode: HostCode,
    },
}

impl<'m> FuncInst<'m> {
    pub fn type_(&self) -> FuncType<'m> {
        match self {
            FuncInst::Guest { type_, .. } | FuncInst::Host { type_ } => *type_,
        }
    }
}

#[derive(Debug)]
pub struct TableInst {
    type_: TableType,
    elem: Vec<Val>,
}

// Invariants:
// - type_.min <= size <= type_.max
// - data.len() <= type_.max * 64K
// Accessing data between data.len() and size * 64K triggers a trap.
#[derive(Debug)]
pub struct MemInst<'m> {
    type_: MemType,
    data: &'m mut [u8],
    size: u32,
}

#[derive(Debug)]
pub struct GlobalInst {
    type_: GlobalType,
    value: Val,
}

#[derive(Debug)]
pub struct ElemInst {
    type_: RefType,
    elem: Vec<Val>, // TODO: Should probably be done lazily.
}

#[derive(Debug)]
pub struct DataInst<'m> {
    data: &'m [u8],
}

#[derive(Debug)]
pub struct ExportInst<'m> {
    name: &'m str,
    value: ExternVal<'m>,
}

#[derive(Debug)]
pub enum ExternVal<'m> {
    Func(Addr<FuncInst<'m>>),
    Table(Addr<TableInst>),
    Mem(Addr<MemInst<'m>>),
    Global(Addr<GlobalInst>),
}

impl<'m> ExternVal<'m> {
    // TODO: This should actually return Invalid instead of panic.
    pub fn type_(&self, store: &Store) -> ExternType<'m> {
        match *self {
            ExternVal::Func(a) => ExternType::Func(store.get(a).type_()),
            ExternVal::Table(a) => ExternType::Table(store.get(a).type_),
            ExternVal::Mem(a) => ExternType::Mem(store.get(a).type_),
            ExternVal::Global(a) => ExternType::Global(store.get(a).type_),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Addr<T> {
    addr: usize,
    _type: PhantomData<T>,
}

trait Data<'m>: Sized {
    fn list<'a>(store: &'a Store<'m>) -> &'a [Self];
    fn list_mut<'a>(store: &'a mut Store<'m>) -> &'a mut Vec<Self>;
}

macro_rules! impl_data {
    ($n:ident: $t:ty) => {
        impl<'m> Data<'m> for $t {
            fn list<'a>(store: &'a Store<'m>) -> &'a [Self] {
                &store.$n
            }
            fn list_mut<'a>(store: &'a mut Store<'m>) -> &'a mut Vec<Self> {
                &mut store.$n
            }
        }
    };
}
impl_data!(funcs: FuncInst<'m>);
impl_data!(tables: TableInst);
impl_data!(mems: MemInst<'m>);
impl_data!(globals: GlobalInst);
impl_data!(modules: ModuleInst<'m>);

impl<'m> Store<'m> {
    pub fn get<T: Data<'m>>(&self, p: Addr<T>) -> &T {
        &<T as Data>::list(self)[p.addr]
    }

    pub fn insert<T: Data<'m>>(&mut self, x: T) -> Addr<T> {
        let xs = <T as Data>::list_mut(self);
        let addr = Addr { addr: xs.len(), _type: PhantomData };
        xs.push(x);
        addr
    }
}

#[derive(Debug)]
pub enum Stack<'m> {
    Frames(Vec<Frame<'m>>),
    Result(Vec<Val>),
    Trap,
}

#[derive(Debug)]
pub struct Frame<'m> {
    arity: usize,
    locals: Vec<Val>,
    module: Addr<ModuleInst<'m>>,
    labels: Vec<Label<'m>>,
}

#[derive(Debug)]
pub struct Label<'m> {
    arity: usize,
    cont: Option<&'m [u8]>,
    values: Vec<Val>,
}

#[derive(Debug)]
struct StoreSingleton;
impl Default for StoreSingleton {
    fn default() -> Self {
        static EXISTS: AtomicBool = AtomicBool::new(false);
        assert!(!EXISTS.swap(true, Ordering::SeqCst));
        Self
    }
}
