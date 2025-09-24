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

// TODO: Some toctou could be used instead of panic.
use alloc::vec;
use alloc::vec::Vec;

use wasefire_common::id::UniqueId;

use crate::cursor::*;
use crate::error::*;
use crate::module::*;
use crate::side_table::*;
use crate::syntax::*;
use crate::toctou::*;
use crate::*;

pub const MEMORY_ALIGN: usize = 16;

/// Runtime values.
// TODO: Introduce untyped values? (to save the tag)
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Val {
    I32(u32),
    I64(u64),
    #[cfg(feature = "float-types")]
    F32(u32),
    #[cfg(feature = "float-types")]
    F64(u64),
    #[cfg(feature = "vector-types")]
    V128(u128),
    Null(RefType),
    Ref(Ptr),
    RefExtern(usize),
}

static STORE_ID: UniqueId = UniqueId::new();

/// Runtime store.
// We cannot GC by design. Vectors can only grow.
#[derive(Debug)]
pub struct Store<'m> {
    id: usize,
    insts: Vec<Instance<'m>>,
    // TODO: This should be an Linker struct that can be shared between stores. The FuncType can be
    // reconstructed on demand (only counts can be stored).
    funcs: Vec<(HostName<'m>, FuncType<'m>)>,
    // When present, contains a module name and a length. In that case, any unresolved imported
    // function for that module name is appended to funcs (one per type, so the function name is
    // the name of the first unresolved function of that type). The length of resolvable host
    // functions in `funcs` is stored to limit normal linking to that part.
    func_default: Option<(&'m str, usize)>,
    threads: Vec<Continuation<'m>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct HostName<'m> {
    module: &'m str,
    name: &'m str,
}

/// Identifies a store.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StoreId(usize);

/// Identifies an instance within a store.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstId {
    store_id: usize,
    inst_id: usize,
}

impl InstId {
    /// Returns the identifier of the store of this instance.
    pub fn store_id(self) -> StoreId {
        StoreId(self.store_id)
    }
}

/// Store wrapper when calling into the host.
#[derive(Debug)]
// Invariant that there is at least one thread.
pub struct Call<'a, 'm> {
    store: &'a mut Store<'m>,
}

impl Default for Store<'_> {
    fn default() -> Self {
        Self {
            id: STORE_ID.next() as usize,
            insts: vec![],
            funcs: vec![],
            func_default: None,
            threads: vec![],
        }
    }
}

impl<'m> Store<'m> {
    /// Returns the identifier of this store.
    pub fn id(&self) -> StoreId {
        StoreId(self.id)
    }

    /// Instantiates a valid module in this store.
    ///
    /// The memory is not dynamically allocated and must thus be provided. It is not necessary for
    /// the memory length to be a multiple of 64kB. Execution will trap if the module tries to
    /// access part of the memory that does not exist.
    pub fn instantiate(
        &mut self, module: Module<'m>, memory: &'m mut [u8],
    ) -> Result<InstId, Error> {
        let inst_id = self.insts.len();
        self.insts.push(Instance::new(module));
        for import in self.last_inst().module.imports() {
            let type_ = import.type_(&self.last_inst().module);
            let id = self.resolve(&import, type_)?;
            match import.desc {
                ImportDesc::Func(_) => self.last_inst().funcs.ext.push(id),
                ImportDesc::Table(_) => self.last_inst().tables.ext.push(id),
                ImportDesc::Mem(_) => self.last_inst().mems.ext.push(id),
                ImportDesc::Global(_) => self.last_inst().globals.ext.push(id),
            }
        }
        if let Some(mut parser) = self.last_inst().module.section(SectionId::Table) {
            for _ in 0 .. parser.parse_vec().into_ok() {
                (self.last_inst().tables.int).push(Table::new(parser.parse_tabletype().into_ok()));
            }
        }
        if let Some(mut parser) = self.last_inst().module.section(SectionId::Memory) {
            match parser.parse_vec().into_ok() {
                0 => (),
                1 => {
                    let limits = parser.parse_memtype().into_ok();
                    self.last_inst().mems.int.init(memory, limits)?;
                }
                _ => unimplemented!(),
            }
        }
        if let Some(mut parser) = self.last_inst().module.section(SectionId::Global) {
            for _ in 0 .. parser.parse_vec().into_ok() {
                parser.parse_globaltype().into_ok();
                let value = Thread::const_expr(self, inst_id, &mut parser);
                self.last_inst().globals.int.push(Global::new(value));
            }
        }
        if let Some(mut parser) = self.last_inst().module.section(SectionId::Element) {
            for _ in 0 .. parser.parse_vec().into_ok() {
                // TODO: This is inefficient because we only need init for active segments.
                let mut elem = ComputeElem::new(self, inst_id);
                parser.parse_elem(&mut elem).into_ok();
                let ComputeElem { mode, init, .. } = elem;
                let drop = match mode {
                    ElemMode::Passive => false,
                    ElemMode::Active { table, offset } => {
                        let n = init.len();
                        let table = self.table(inst_id, table);
                        table_init(offset, 0, n, table, &init)?;
                        true
                    }
                    ElemMode::Declarative => true,
                };
                self.last_inst().elems.push(drop);
            }
        }
        if let Some(mut parser) = self.last_inst().module.section(SectionId::Data) {
            for _ in 0 .. parser.parse_vec().into_ok() {
                let mut data = ComputeData::new(self, inst_id);
                parser.parse_data(&mut data).into_ok();
                let ComputeData { mode, init, .. } = data;
                let drop = match mode {
                    DataMode::Passive => false,
                    DataMode::Active { memory, offset } => {
                        let n = init.len();
                        let memory = self.mem(inst_id, memory);
                        memory_init(offset, 0, n, memory, init)?;
                        true
                    }
                };
                self.last_inst().datas.push(drop);
            }
        }
        if let Some(mut parser) = self.last_inst().module.section(SectionId::Start) {
            let x = parser.parse_funcidx().into_ok();
            let ptr = self.func_ptr(inst_id, x);
            let inst_id = ptr.instance().unwrap_wasm();
            let (mut parser, side_table) = self.insts[inst_id].module.func(ptr.index());
            let mut locals = Vec::new();
            append_locals(&mut parser, &mut locals);
            let ret = Parser::default();
            let thread = Thread::new(parser, Frame::new(inst_id, 0, ret, locals, side_table, 0));
            let result = thread.run(self)?;
            assert!(matches!(result, RunResult::Done(x) if x.is_empty()));
        }
        Ok(InstId { store_id: self.id, inst_id })
    }

    /// Invokes a function in an instance provided its name.
    ///
    /// If a function was already running, it will resume once the function being called terminates.
    /// In other words, execution satisfies a stack property. Without this, the stack of the module
    /// may be corrupted.
    pub fn invoke<'a>(
        &'a mut self, inst: InstId, name: &str, args: Vec<Val>,
    ) -> Result<RunResult<'a, 'm>, Error> {
        let inst_id = self.inst_id(inst)?;
        let inst = &self.insts[inst_id];
        let ptr = match inst.module.export(name).ok_or_else(not_found)? {
            ExportDesc::Func(x) => inst.funcs.ptr(inst_id, x),
            _ => return Err(Error::Invalid),
        };
        let inst_id = ptr.instance().unwrap_wasm();
        let inst = &self.insts[inst_id];
        let x = ptr.index();
        let t = inst.module.func_type(x);
        let (mut parser, side_table) = inst.module.func(x);
        check_types(&t.params, &args)?;
        let mut locals = args;
        append_locals(&mut parser, &mut locals);
        let ret = Parser::default();
        let frame = Frame::new(inst_id, t.results.len(), ret, locals, side_table, 0);
        Thread::new(parser, frame).run(self)
    }

    /// Returns the value of a global of an instance.
    pub fn get_global(&mut self, inst: InstId, name: &str) -> Result<Val, Error> {
        let inst_id = self.inst_id(inst)?;
        let inst = &self.insts[inst_id];
        let ptr = match inst.module.export(name).ok_or_else(not_found)? {
            ExportDesc::Global(x) => inst.globals.ptr(inst_id, x),
            _ => return Err(Error::Invalid),
        };
        let inst_id = ptr.instance().unwrap_wasm();
        let x = ptr.index() as usize;
        Ok(self.insts[inst_id].globals.int[x].value)
    }

    /// Sets the name of an instance.
    pub fn set_name(&mut self, inst: InstId, name: &'m str) -> Result<(), Error> {
        let inst_id = self.inst_id(inst)?;
        self.insts[inst_id].name = name;
        Ok(())
    }

    /// Links a host function provided its signature.
    ///
    /// This is a convenient wrapper around [`Self::link_func_custom()`] for functions which
    /// parameter and return types are only `i32`. The `params` and `results` parameters describe
    /// how many `i32` are taken as parameters and returned as results respectively.
    pub fn link_func(
        &mut self, module: &'m str, name: &'m str, params: usize, results: usize,
    ) -> Result<(), Error> {
        static TYPES: &[ValType] = &[ValType::I32; 8];
        check(params <= TYPES.len() && results <= TYPES.len())?;
        let type_ = FuncType { params: TYPES[.. params].into(), results: TYPES[.. results].into() };
        self.link_func_custom(module, name, type_)
    }

    /// Enables linking unresolved imported function for the given module.
    ///
    /// For each unresolved imported function type that returns exactly one `i32`, a fake host
    /// function is created (as if `link_func` was used). As such, out-of-bound indices with respect
    /// to real calls to [`Self::link_func()`] represent such fake host functions. Callers must thus
    /// expect and support out-of-bound indices returned by [`Call::index()`].
    ///
    /// This function can be called at most once, and once called `link_func` cannot be called.
    pub fn link_func_default(&mut self, module: &'m str) -> Result<(), Error> {
        check(self.func_default.is_none())?;
        self.func_default = Some((module, self.funcs.len()));
        Ok(())
    }

    /// Links a host function provided its signature.
    ///
    /// Note that the order in which functions are linked defines their index. The first linked
    /// function has index 0, the second has index 1, etc. This index is used when a module calls in
    /// the host to identify the function.
    pub fn link_func_custom(
        &mut self, module: &'m str, name: &'m str, type_: FuncType<'m>,
    ) -> Result<(), Error> {
        let name = HostName { module, name };
        check(self.func_default.is_none())?;
        check(self.insts.is_empty())?;
        check(self.funcs.last().is_none_or(|x| x.0 < name))?;
        self.funcs.push((name, type_));
        Ok(())
    }

    /// Returns the call in the host, if any.
    ///
    /// This function returns `None` if nothing is running.
    // NOTE: This is like poll. Could be called next.
    pub fn last_call(&mut self) -> Option<Call<'_, 'm>> {
        if self.threads.is_empty() { None } else { Some(Call { store: self }) }
    }

    /// Returns the memory of the given instance.
    pub fn memory(&mut self, inst: InstId) -> Result<&mut [u8], Error> {
        check(self.id == inst.store_id)?;
        Ok(self.mem(inst.inst_id, 0).data)
    }
}

impl<'a, 'm> Call<'a, 'm> {
    /// Returns the index of the host function being called.
    pub fn index(&self) -> usize {
        self.cont().index
    }

    /// Returns the arguments to the host function being called.
    pub fn args(&self) -> &[Val] {
        &self.cont().args
    }

    /// Returns the identifier of the instance calling the host.
    pub fn inst(&self) -> InstId {
        self.cont().thread.inst(self.store)
    }

    /// Returns the memory of the instance calling the host.
    pub fn mem(self) -> &'a mut [u8] {
        self.store.mem(self.inst().inst_id, 0).data
    }

    /// Returns the memory of the instance calling the host.
    pub fn mem_mut(&mut self) -> &mut [u8] {
        self.store.mem(self.inst().inst_id, 0).data
    }

    /// Resumes execution with the results from the host.
    pub fn resume(self, results: &[Val]) -> Result<RunResult<'a, 'm>, Error> {
        let Continuation { mut thread, arity, .. } = self.store.threads.pop().unwrap();
        check(results.len() == arity)?;
        thread.push_values(results);
        thread.run(self.store)
    }

    fn cont(&self) -> &Continuation<'_> {
        self.store.threads.last().unwrap()
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Ptr(u32);

impl core::fmt::Debug for Ptr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Ptr")
            .field("instance", &self.instance())
            .field("index", &self.index())
            .finish()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Side {
    Host,
    Wasm(usize),
}

impl Side {
    fn into_repr(self) -> usize {
        match self {
            Side::Host => 0,
            Side::Wasm(x) => 1 + x,
        }
    }

    fn from_repr(x: usize) -> Self {
        match x.checked_sub(1) {
            None => Side::Host,
            Some(x) => Side::Wasm(x),
        }
    }

    fn unwrap_wasm(self) -> usize {
        match self {
            Side::Host => unreachable!(),
            Side::Wasm(x) => x,
        }
    }
}

impl Ptr {
    // 4k modules with 1M indices (per component)
    const INDEX_BITS: usize = 20;
    const INDEX_MASK: u32 = (1 << Self::INDEX_BITS) - 1;
    const INST_MASK: u32 = (1 << (32 - Self::INDEX_BITS)) - 1;

    fn new(inst: Side, idx: u32) -> Self {
        let inst = inst.into_repr() as u32;
        assert_eq!(inst & !Self::INST_MASK, 0);
        assert_eq!(idx & !Self::INDEX_MASK, 0);
        Self((inst << Self::INDEX_BITS) | idx)
    }

    fn instance(self) -> Side {
        Side::from_repr((self.0 >> Self::INDEX_BITS) as usize)
    }

    fn index(self) -> u32 {
        self.0 & Self::INDEX_MASK
    }
}

impl core::fmt::Debug for Val {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Val::I32(x) => write!(f, "{x}"),
            Val::I64(x) => write!(f, "{x}"),
            #[cfg(feature = "float-types")]
            Val::F32(x) => write!(f, "{}", f32::from_bits(x)),
            #[cfg(feature = "float-types")]
            Val::F64(x) => write!(f, "{}", f64::from_bits(x)),
            #[cfg(feature = "vector-types")]
            Val::V128(_) => todo!(),
            Val::Null(_) => write!(f, "null"),
            Val::Ref(p) => write!(f, "ref@{:?}:{}", p.instance(), p.index()),
            Val::RefExtern(p) => write!(f, "ext@{p}"),
        }
    }
}

impl Val {
    pub fn type_(self) -> ValType {
        match self {
            Val::I32(_) => ValType::I32,
            Val::I64(_) => ValType::I64,
            #[cfg(feature = "float-types")]
            Val::F32(_) => ValType::F32,
            #[cfg(feature = "float-types")]
            Val::F64(_) => ValType::F64,
            #[cfg(feature = "vector-types")]
            Val::V128(_) => ValType::V128,
            Val::Null(t) => t.into(),
            Val::Ref(_) => ValType::FuncRef,
            Val::RefExtern(_) => ValType::ExternRef,
        }
    }
}

#[derive(Debug)]
struct Instance<'m> {
    // TODO: Make sure names are unique. Empty name means exports are not linked.
    name: &'m str,
    module: Module<'m>,
    funcs: Component<()>,
    tables: Component<Vec<Table>>,
    mems: Component<Memory<'m>>,
    globals: Component<Vec<Global>>,
    elems: Vec<bool>, // whether the elem segment is dropped
    datas: Vec<bool>, // whether the data segment is dropped
}

#[derive(Debug)]
struct Thread<'m> {
    parser: Parser<'m>,
    frames: Vec<Frame<'m>>,
    // TODO: Consider the performance tradeoff between keeping the locals in and out of the value
    // stack. See the comments in PR #605 for more details.
    values: Vec<Val>,
}

/// Runtime result.
#[derive(Debug)]
pub enum RunResult<'a, 'm> {
    /// Execution terminated successfully with those values.
    Done(Vec<Val>),

    /// Execution is calling into the host.
    Host(Call<'a, 'm>),
}

/// Runtime result without host call information.
#[derive(Debug)]
pub enum RunAnswer {
    Done(Vec<Val>),
    Host,
}

impl RunResult<'_, '_> {
    pub fn forget(self) -> RunAnswer {
        match self {
            RunResult::Done(result) => RunAnswer::Done(result),
            RunResult::Host(_) => RunAnswer::Host,
        }
    }
}

#[derive(Debug)]
struct Continuation<'m> {
    thread: Thread<'m>,
    index: usize,
    args: Vec<Val>,
    arity: usize,
}

impl<'m> Store<'m> {
    fn last_inst(&mut self) -> &mut Instance<'m> {
        self.insts.last_mut().unwrap()
    }

    fn inst_id(&self, inst: InstId) -> Result<usize, Error> {
        check(self.id == inst.store_id)?;
        Ok(inst.inst_id)
    }

    fn resolve_inst(&self, name: &str) -> Result<usize, Error> {
        self.insts.iter().position(|x| x.name == name).ok_or_else(not_found)
    }

    fn func_type(&self, ptr: Ptr) -> FuncType<'m> {
        match ptr.instance() {
            Side::Host => self.funcs[ptr.index() as usize].1,
            Side::Wasm(x) => self.insts[x].module.func_type(ptr.index()),
        }
    }

    fn func_ptr(&self, inst_id: usize, x: FuncIdx) -> Ptr {
        self.insts[inst_id].funcs.ptr(inst_id, x)
    }

    fn table_ptr(&self, inst_id: usize, x: TableIdx) -> Ptr {
        self.insts[inst_id].tables.ptr(inst_id, x)
    }

    fn mem_ptr(&self, inst_id: usize, x: MemIdx) -> Ptr {
        self.insts[inst_id].mems.ptr(inst_id, x)
    }

    fn global_ptr(&self, inst_id: usize, x: GlobalIdx) -> Ptr {
        self.insts[inst_id].globals.ptr(inst_id, x)
    }

    fn table(&mut self, inst_id: usize, x: TableIdx) -> &mut Table {
        let ptr = self.table_ptr(inst_id, x);
        let x = ptr.instance().unwrap_wasm();
        &mut self.insts[x].tables.int[ptr.index() as usize]
    }

    fn mem(&mut self, inst_id: usize, x: MemIdx) -> &mut Memory<'m> {
        let ptr = self.mem_ptr(inst_id, x);
        let x = ptr.instance().unwrap_wasm();
        assert_eq!(ptr.index(), 0);
        &mut self.insts[x].mems.int
    }

    fn global(&mut self, inst_id: usize, x: GlobalIdx) -> &mut Global {
        let ptr = self.global_ptr(inst_id, x);
        let x = ptr.instance().unwrap_wasm();
        &mut self.insts[x].globals.int[ptr.index() as usize]
    }

    fn resolve(&mut self, import: &Import<'m>, imp_type_: ExternType<'m>) -> Result<Ptr, Error> {
        let host_name = HostName { module: import.module, name: import.name };
        let mut found = None;
        let funcs_len = match self.func_default {
            Some((_, x)) => x,
            None => self.funcs.len(),
        };
        if let Ok(x) = self.funcs[.. funcs_len].binary_search_by(|x| x.0.cmp(&host_name)) {
            found = Some((Ptr::new(Side::Host, x as u32), ExternType::Func(self.funcs[x].1)));
        } else if matches!(imp_type_, ExternType::Func(x) if *x.results == [ValType::I32])
            && matches!(self.func_default, Some((x, _)) if x == host_name.module)
        {
            let type_ = match imp_type_ {
                ExternType::Func(x) => x,
                _ => unreachable!(),
            };
            let idx = match self.funcs[funcs_len ..].iter().position(|x| x.1 == type_) {
                Some(x) => funcs_len + x,
                None => {
                    let idx = self.funcs.len();
                    self.funcs.push((host_name, type_));
                    idx
                }
            };
            found = Some((Ptr::new(Side::Host, idx as u32), ExternType::Func(type_)));
        } else {
            let inst_id = self.resolve_inst(import.module)?;
            let inst = &self.insts[inst_id];
            if let Some(mut parser) = inst.module.section(SectionId::Export) {
                for _ in 0 .. parser.parse_vec().into_ok() {
                    let name = parser.parse_name().into_ok();
                    let desc = parser.parse_exportdesc().into_ok();
                    if name != import.name {
                        continue;
                    }
                    found = Some(match desc {
                        ExportDesc::Func(x) => {
                            let ptr = self.func_ptr(inst_id, x);
                            (ptr, ExternType::Func(self.func_type(ptr)))
                        }
                        ExportDesc::Table(x) => {
                            let ptr = self.table_ptr(inst_id, x);
                            let inst = &self.insts[ptr.instance().unwrap_wasm()];
                            let mut t = inst.module.table_type(ptr.index());
                            t.limits.min = inst.tables.int[ptr.index() as usize].size();
                            (ptr, ExternType::Table(t))
                        }
                        ExportDesc::Mem(x) => {
                            let ptr = self.mem_ptr(inst_id, x);
                            let inst = &self.insts[ptr.instance().unwrap_wasm()];
                            let mut t = inst.module.mem_type(ptr.index());
                            assert_eq!(ptr.index(), 0);
                            t.min = inst.mems.int.size();
                            (ptr, ExternType::Mem(t))
                        }
                        ExportDesc::Global(x) => {
                            let ptr = self.global_ptr(inst_id, x);
                            let inst = &self.insts[ptr.instance().unwrap_wasm()];
                            (ptr, ExternType::Global(inst.module.global_type(ptr.index())))
                        }
                    });
                    break;
                }
            }
        }
        let (ptr, ext_type_) = found.ok_or_else(not_found)?;
        if ext_type_.matches(&imp_type_) { Ok(ptr) } else { Err(not_found()) }
    }
}

impl<'m> Instance<'m> {
    fn new(module: Module<'m>) -> Self {
        Instance {
            name: "",
            module,
            funcs: Component::default(),
            tables: Component::default(),
            mems: Component::default(),
            globals: Component::default(),
            elems: Vec::new(),
            datas: Vec::new(),
        }
    }
}

#[derive(Debug)]
enum ElemMode {
    Passive,
    Active { table: TableIdx, offset: usize },
    Declarative,
}

struct ComputeElem<'a, 'm> {
    store: &'a mut Store<'m>,
    inst_id: usize,
    mode: ElemMode,
    init: Vec<Val>,
}

impl<'a, 'm> ComputeElem<'a, 'm> {
    fn new(store: &'a mut Store<'m>, inst_id: usize) -> Self {
        Self { store, inst_id, mode: ElemMode::Passive, init: Vec::new() }
    }
}

impl<'m> parser::ParseElem<'m, Use> for ComputeElem<'_, 'm> {
    fn mode(&mut self, mode: parser::ElemMode<'_, 'm, Use>) -> MResult<(), Use> {
        let mode = match mode {
            parser::ElemMode::Passive => ElemMode::Passive,
            parser::ElemMode::Active { table, offset } => ElemMode::Active {
                table,
                offset: Thread::const_expr(self.store, self.inst_id, offset).unwrap_i32() as usize,
            },
            parser::ElemMode::Declarative => ElemMode::Declarative,
        };
        self.mode = mode;
        Ok(())
    }

    fn init_funcidx(&mut self, x: FuncIdx) -> MResult<(), Use> {
        self.init.push(Val::Ref(self.store.func_ptr(self.inst_id, x)));
        Ok(())
    }

    fn init_expr(&mut self, parser: &mut Parser<'m>) -> MResult<(), Use> {
        self.init.push(Thread::const_expr(self.store, self.inst_id, parser));
        Ok(())
    }
}

#[derive(Debug)]
enum DataMode {
    Passive,
    Active { memory: MemIdx, offset: usize },
}

struct ComputeData<'a, 'm> {
    store: &'a mut Store<'m>,
    inst_id: usize,
    mode: DataMode,
    init: &'m [u8],
}

impl<'a, 'm> ComputeData<'a, 'm> {
    fn new(store: &'a mut Store<'m>, inst_id: usize) -> Self {
        Self { store, inst_id, mode: DataMode::Passive, init: &[] }
    }
}

impl<'m> parser::ParseData<'m, Use> for ComputeData<'_, 'm> {
    fn mode(&mut self, mode: parser::DataMode<'_, 'm, Use>) -> MResult<(), Use> {
        let mode = match mode {
            parser::DataMode::Passive => DataMode::Passive,
            parser::DataMode::Active { memory, offset } => DataMode::Active {
                memory,
                offset: Thread::const_expr(self.store, self.inst_id, offset).unwrap_i32() as usize,
            },
        };
        self.mode = mode;
        Ok(())
    }

    fn init(&mut self, init: &'m [u8]) -> MResult<(), Use> {
        self.init = init;
        Ok(())
    }
}

#[derive(Debug)]
struct Table {
    max: u32,
    elems: Vec<Val>,
}

#[derive(Debug)]
struct Global {
    value: Val,
}

enum ThreadResult<'m> {
    Continue(Thread<'m>),
    Done(Vec<Val>),
    Host,
}

impl<'m> Thread<'m> {
    fn new(parser: Parser<'m>, frame: Frame<'m>) -> Thread<'m> {
        Thread { parser, frames: vec![frame], values: vec![] }
    }

    fn const_expr(store: &mut Store<'m>, inst_id: usize, mut_parser: &mut Parser<'m>) -> Val {
        let mut thread = Thread::new(
            mut_parser.clone(),
            Frame::new(inst_id, 1, Parser::default(), Vec::new(), Cursor::default(), 0),
        );
        let (state, results) = loop {
            let s = thread.parser.save();
            match thread.step(store).unwrap() {
                ThreadResult::Continue(x) => thread = x,
                ThreadResult::Done(x) => break (s, x),
                ThreadResult::Host => unreachable!(),
            }
        };
        unsafe { mut_parser.restore(state) };
        let instr = mut_parser.parse_instr().into_ok();
        debug_assert_eq!(instr, Instr::End);
        debug_assert_eq!(results.len(), 1);
        results[0]
    }

    fn run<'a>(mut self, store: &'a mut Store<'m>) -> Result<RunResult<'a, 'm>, Error> {
        loop {
            // TODO: When trapping, we could return some CoreDump<'m> that contains the Thread<'m>.
            // This permits to dump the frames.
            match self.step(store)? {
                ThreadResult::Continue(x) => self = x,
                ThreadResult::Done(x) => return Ok(RunResult::Done(x)),
                ThreadResult::Host => return Ok(RunResult::Host(Call { store })),
            }
        }
    }

    fn step(mut self, store: &mut Store<'m>) -> Result<ThreadResult<'m>, Error> {
        use Instr::*;
        let inst_id = self.frame().inst_id;
        let inst = &mut store.insts[inst_id];
        match self.parser.parse_instr().into_ok() {
            Unreachable => return Err(trap()),
            Nop => (),
            Block(_) => self.push_label(),
            Loop(_) => self.push_label(),
            If(_) => match self.pop_value().unwrap_i32() {
                0 => {
                    self.take_jump(0);
                    self.push_label();
                }
                _ => {
                    self.frame().skip_jump();
                    self.push_label();
                }
            },
            Else => {
                self.take_jump(0);
                return Ok(self.exit_label());
            }
            End => return Ok(self.exit_label()),
            Br(l) => return Ok(self.pop_label(l, 0)),
            BrIf(l) => {
                if self.pop_value().unwrap_i32() != 0 {
                    return Ok(self.pop_label(l, 0));
                }
                self.frame().skip_jump();
            }
            BrTable(ls, ln) => {
                let i = self.pop_value().unwrap_i32() as usize;
                let (l, offset) = match ls.get(i) {
                    None => (ln, 0),
                    Some(&li) => (li, i + 1),
                };
                return Ok(self.pop_label(l, offset));
            }
            Return => return Ok(self.exit_frame()),
            Call(x) => return self.invoke(store, store.func_ptr(inst_id, x)),
            CallIndirect(x, y) => {
                let i = self.pop_value().unwrap_i32();
                let x = match store.table(inst_id, x).elems.get(i as usize) {
                    None | Some(Val::Null(_)) => return Err(trap()),
                    Some(x) => x.unwrap_ref(),
                };
                if store.func_type(x) != store.insts[inst_id].module.types()[y as usize] {
                    return Err(trap());
                }
                return self.invoke(store, x);
            }
            Drop => drop(self.pop_value()),
            Select(_) => {
                let c = self.pop_value().unwrap_i32();
                let v2 = self.pop_value();
                let v1 = self.pop_value();
                self.push_value(match c {
                    0 => v2,
                    _ => v1,
                });
            }
            LocalGet(x) => {
                let v = self.frame().locals[x as usize];
                self.push_value(v);
            }
            LocalSet(x) => {
                let v = self.pop_value();
                self.frame().locals[x as usize] = v;
            }
            LocalTee(x) => {
                let v = self.peek_value();
                self.frame().locals[x as usize] = v;
            }
            GlobalGet(x) => self.push_value(store.global(inst_id, x).value),
            GlobalSet(x) => store.global(inst_id, x).value = self.pop_value(),
            TableGet(x) => {
                let i = self.pop_value().unwrap_i32();
                let v = *store.table(inst_id, x).elems.get(i as usize).ok_or_else(trap)?;
                self.push_value(v);
            }
            TableSet(x) => {
                let val = self.pop_value();
                let i = self.pop_value().unwrap_i32();
                let v = store.table(inst_id, x).elems.get_mut(i as usize).ok_or_else(trap)?;
                *v = val;
            }
            ILoad(n, m) => self.load(store.mem(inst_id, 0), NumType::i(n), n.into(), Sx::U, m)?,
            #[cfg(feature = "float-types")]
            FLoad(n, m) => self.load(store.mem(inst_id, 0), NumType::f(n), n.into(), Sx::U, m)?,
            ILoad_(b, s, m) => {
                self.load(store.mem(inst_id, 0), NumType::i(b.into()), b.into(), s, m)?
            }
            IStore(n, m) => self.store(store.mem(inst_id, 0), NumType::i(n), n.into(), m)?,
            #[cfg(feature = "float-types")]
            FStore(n, m) => self.store(store.mem(inst_id, 0), NumType::f(n), n.into(), m)?,
            IStore_(b, m) => {
                self.store(store.mem(inst_id, 0), NumType::i(b.into()), b.into(), m)?
            }
            MemorySize => self.push_value(Val::I32(store.mem(inst_id, 0).size())),
            MemoryGrow => {
                let n = self.pop_value().unwrap_i32();
                self.push_value(Val::I32(grow(store.mem(inst_id, 0), n, ())));
            }
            I32Const(c) => self.push_value(Val::I32(c)),
            I64Const(c) => self.push_value(Val::I64(c)),
            #[cfg(feature = "float-types")]
            F32Const(c) => self.push_value(Val::F32(c)),
            #[cfg(feature = "float-types")]
            F64Const(c) => self.push_value(Val::F64(c)),
            ITestOp(n, op) => self.itestop(n, op),
            IRelOp(n, op) => self.irelop(n, op),
            #[cfg(feature = "float-types")]
            FRelOp(n, op) => self.frelop(n, op),
            IUnOp(n, op) => self.iunop(n, op)?,
            #[cfg(feature = "float-types")]
            FUnOp(n, op) => self.funop(n, op),
            IBinOp(n, op) => self.ibinop(n, op)?,
            #[cfg(feature = "float-types")]
            FBinOp(n, op) => self.fbinop(n, op),
            CvtOp(op) => self.cvtop(op)?,
            IExtend(b) => self.extend(b),
            RefNull(t) => self.push_value(Val::Null(t)),
            RefIsNull => {
                let c = matches!(self.pop_value(), Val::Null(_)) as u32;
                self.push_value(Val::I32(c));
            }
            RefFunc(x) => self.push_value(Val::Ref(store.func_ptr(inst_id, x))),
            MemoryInit(x) => {
                let n = self.pop_value().unwrap_i32() as usize;
                let s = self.pop_value().unwrap_i32() as usize;
                let d = self.pop_value().unwrap_i32() as usize;
                let data = if inst.datas[x as usize] {
                    &[]
                } else {
                    let mut parser = inst.module.data(x);
                    let mut data = ComputeData::new(store, inst_id);
                    parser.parse_data(&mut data).into_ok();
                    data.init
                };
                let mem = store.mem(inst_id, 0);
                memory_init(d, s, n, mem, data)?;
            }
            DataDrop(x) => inst.datas[x as usize] = true,
            MemoryCopy => {
                let n = self.pop_value().unwrap_i32() as usize;
                let s = self.pop_value().unwrap_i32() as usize;
                let d = self.pop_value().unwrap_i32() as usize;
                let mem = store.mem(inst_id, 0);
                if core::cmp::max(s, d).checked_add(n).is_none_or(|x| x > mem.len() as usize) {
                    return Err(trap());
                }
                mem.data.copy_within(s .. s + n, d);
            }
            MemoryFill => {
                let n = self.pop_value().unwrap_i32() as usize;
                let val = self.pop_value().unwrap_i32() as u8;
                let d = self.pop_value().unwrap_i32() as usize;
                let mem = store.mem(inst_id, 0);
                if d.checked_add(n).is_none_or(|x| x > mem.len() as usize) {
                    memory_too_small(d, n, mem);
                    return Err(trap());
                }
                mem.data[d ..][.. n].fill(val);
            }
            TableInit(x, y) => {
                let n = self.pop_value().unwrap_i32() as usize;
                let s = self.pop_value().unwrap_i32() as usize;
                let d = self.pop_value().unwrap_i32() as usize;
                let elems = if inst.elems[y as usize] {
                    Vec::new()
                } else {
                    let mut parser = inst.module.elem(y);
                    let mut elems = ComputeElem::new(store, inst_id);
                    parser.parse_elem(&mut elems).into_ok();
                    elems.init
                };
                let table = store.table(inst_id, x);
                table_init(d, s, n, table, &elems)?;
            }
            ElemDrop(x) => inst.elems[x as usize] = true,
            TableCopy(x, y) => {
                let n = self.pop_value().unwrap_i32() as usize;
                let s = self.pop_value().unwrap_i32() as usize;
                let d = self.pop_value().unwrap_i32() as usize;
                let sn = s.checked_add(n).ok_or_else(trap)?;
                let dn = d.checked_add(n).ok_or_else(trap)?;
                // TODO: This is not efficient.
                let ys = store.table(inst_id, y).elems.get(s .. sn).ok_or_else(trap)?.to_vec();
                let xs = store.table(inst_id, x).elems.get_mut(d .. dn).ok_or_else(trap)?;
                xs.copy_from_slice(&ys);
            }
            TableGrow(x) => {
                let n = self.pop_value().unwrap_i32();
                let val = self.pop_value();
                let table = store.table(inst_id, x);
                self.push_value(Val::I32(grow(table, n, val)));
            }
            TableSize(x) => self.push_value(Val::I32(store.table(inst_id, x).size())),
            TableFill(x) => {
                let n = self.pop_value().unwrap_i32() as usize;
                let val = self.pop_value();
                let i = self.pop_value().unwrap_i32() as usize;
                let table = store.table(inst_id, x);
                if i.checked_add(n).is_none_or(|x| x > table.elems.len()) {
                    return Err(trap());
                }
                table.elems[i ..][.. n].fill(val);
            }
        }
        Ok(ThreadResult::Continue(self))
    }

    fn inst_id(&self) -> usize {
        self.frames.last().unwrap().inst_id
    }

    fn inst(&self, store: &Store<'m>) -> InstId {
        InstId { store_id: store.id, inst_id: self.inst_id() }
    }

    fn frame(&mut self) -> &mut Frame<'m> {
        self.frames.last_mut().unwrap()
    }

    fn values(&mut self) -> &mut Vec<Val> {
        &mut self.values
    }

    fn peek_value(&mut self) -> Val {
        *self.values().last().unwrap()
    }

    fn push_value(&mut self, value: Val) {
        self.values().push(value);
    }

    fn push_value_or_trap(&mut self, value: Option<Val>) -> Result<(), Error> {
        if let Some(x) = value {
            self.push_value(x);
            Ok(())
        } else {
            Err(trap())
        }
    }

    fn push_values(&mut self, values: &[Val]) {
        self.values().extend_from_slice(values);
    }

    fn pop_value(&mut self) -> Val {
        self.values().pop().unwrap()
    }

    fn pop_values(&mut self, n: usize) -> Vec<Val> {
        let len = self.values().len() - n;
        self.values().split_off(len)
    }

    fn push_label(&mut self) {
        self.frame().labels_cnt += 1;
    }

    fn pop_label(mut self, l: LabelIdx, offset: usize) -> ThreadResult<'m> {
        let frame = self.frame();
        let i = frame.labels_cnt - l as usize - 1;
        if i == 0 {
            return self.exit_frame();
        }
        frame.labels_cnt = i;
        let Ok(BranchTableEntryView { val_cnt, pop_cnt, .. }) =
            frame.side_table.get(offset).view::<Use>();
        let val_pos = self.values().len() - val_cnt as usize;
        self.values().drain(val_pos - pop_cnt as usize .. val_pos);
        self.take_jump(offset);
        ThreadResult::Continue(self)
    }

    fn exit_label(mut self) -> ThreadResult<'m> {
        let frame = self.frame();
        frame.labels_cnt -= 1;
        if frame.labels_cnt == 0 {
            let arity = frame.arity;
            let values = self.pop_values(arity);
            let frame = self.frames.pop().unwrap();
            if self.frames.is_empty() {
                return ThreadResult::Done(values);
            }
            self.parser = frame.ret;
            self.values().extend(values);
        }
        ThreadResult::Continue(self)
    }

    fn exit_frame(mut self) -> ThreadResult<'m> {
        let prev_stack = self.frame().prev_stack;
        let mut values = self.values().split_off(prev_stack);
        let frame = self.frames.pop().unwrap();
        let mid = values.len() - frame.arity;
        if self.frames.is_empty() {
            values.drain(0 .. mid);
            return ThreadResult::Done(values);
        }
        self.parser = frame.ret;
        self.values().extend_from_slice(&values[mid ..]);
        ThreadResult::Continue(self)
    }

    fn take_jump(&mut self, offset: usize) {
        let offset = self.frame().take_jump(offset);
        self.parser.update(offset);
    }

    fn mem_slice<'a>(
        &mut self, mem: &'a mut Memory<'m>, m: MemArg, i: u32, len: usize,
    ) -> Option<&'a mut [u8]> {
        let ea = i.checked_add(m.offset)?;
        if ea.checked_add(len as u32)? > mem.len() {
            memory_too_small(ea as usize, len, mem);
            return None;
        }
        Some(&mut mem.data[ea as usize ..][.. len])
    }

    fn load(
        &mut self, mem: &mut Memory<'m>, t: NumType, n: usize, s: Sx, m: MemArg,
    ) -> Result<(), Error> {
        let i = self.pop_value().unwrap_i32();
        let mem = match self.mem_slice(mem, m, i, n / 8) {
            None => return Err(trap()),
            Some(x) => x,
        };
        macro_rules! convert {
            ($T:ident, $t:ident, $s:ident) => {
                Val::$T($s::from_le_bytes(mem.try_into().unwrap()) as $t)
            };
        }
        let c = match (t, n, s) {
            (NumType::I32, 32, Sx::U) => convert!(I32, u32, u32),
            (NumType::I32, 16, Sx::U) => convert!(I32, u32, u16),
            (NumType::I32, 16, Sx::S) => convert!(I32, u32, i16),
            (NumType::I32, 8, Sx::U) => convert!(I32, u32, u8),
            (NumType::I32, 8, Sx::S) => convert!(I32, u32, i8),
            (NumType::I64, 64, Sx::U) => convert!(I64, u64, u64),
            (NumType::I64, 32, Sx::U) => convert!(I64, u64, u32),
            (NumType::I64, 32, Sx::S) => convert!(I64, u64, i32),
            (NumType::I64, 16, Sx::U) => convert!(I64, u64, u16),
            (NumType::I64, 16, Sx::S) => convert!(I64, u64, i16),
            (NumType::I64, 8, Sx::U) => convert!(I64, u64, u8),
            (NumType::I64, 8, Sx::S) => convert!(I64, u64, i8),
            #[cfg(feature = "float-types")]
            (NumType::F32, 32, _) => convert!(F32, u32, u32),
            #[cfg(feature = "float-types")]
            (NumType::F64, 64, _) => convert!(F64, u64, u64),
            _ => unreachable!(),
        };
        self.push_value(c);
        Ok(())
    }

    fn store(
        &mut self, mem: &mut Memory<'m>, t: NumType, n: usize, m: MemArg,
    ) -> Result<(), Error> {
        let c = self.pop_value();
        let i = self.pop_value().unwrap_i32();
        let mem = match self.mem_slice(mem, m, i, n / 8) {
            None => return Err(trap()),
            Some(x) => x,
        };
        macro_rules! convert {
            ($s:ident, $t:ident) => {
                paste::paste! {
                    mem.copy_from_slice(&(c.[<unwrap_ $s>]() as $t).to_le_bytes())
                }
            };
        }
        match (t, n) {
            (NumType::I32, 32) => convert!(i32, u32),
            (NumType::I32, 16) => convert!(i32, u16),
            (NumType::I32, 8) => convert!(i32, u8),
            (NumType::I64, 64) => convert!(i64, u64),
            (NumType::I64, 32) => convert!(i64, u32),
            (NumType::I64, 16) => convert!(i64, u16),
            (NumType::I64, 8) => convert!(i64, u8),
            #[cfg(feature = "float-types")]
            (NumType::F32, 32) => convert!(f32, u32),
            #[cfg(feature = "float-types")]
            (NumType::F64, 64) => convert!(f64, u64),
            _ => unreachable!(),
        }
        Ok(())
    }

    fn itestop(&mut self, n: Nx, op: ITestOp) {
        let x = self.pop_value();
        let z = match n {
            Nx::N32 => op.n32(x.unwrap_i32()),
            Nx::N64 => op.n64(x.unwrap_i64()),
        };
        self.push_value(Val::I32(z as u32))
    }

    fn irelop(&mut self, n: Nx, op: IRelOp) {
        let y = self.pop_value();
        let x = self.pop_value();
        let z = match n {
            Nx::N32 => op.n32(x.unwrap_i32(), y.unwrap_i32()),
            Nx::N64 => op.n64(x.unwrap_i64(), y.unwrap_i64()),
        };
        self.push_value(Val::I32(z as u32))
    }

    fn iunop(&mut self, n: Nx, op: IUnOp) -> Result<(), Error> {
        let x = self.pop_value();
        let z = try {
            match n {
                Nx::N32 => Val::I32(op.n32(x.unwrap_i32())?),
                Nx::N64 => Val::I64(op.n64(x.unwrap_i64())?),
            }
        };
        self.push_value_or_trap(z)
    }

    fn ibinop(&mut self, n: Nx, op: IBinOp) -> Result<(), Error> {
        let y = self.pop_value();
        let x = self.pop_value();
        let z = try {
            match n {
                Nx::N32 => Val::I32(op.n32(x.unwrap_i32(), y.unwrap_i32())?),
                Nx::N64 => Val::I64(op.n64(x.unwrap_i64(), y.unwrap_i64())?),
            }
        };
        self.push_value_or_trap(z)
    }

    #[cfg(feature = "float-types")]
    fn frelop(&mut self, n: Nx, op: FRelOp) {
        let y = self.pop_value();
        let x = self.pop_value();
        let z = match n {
            Nx::N32 => op.n32(x.unwrap_f32(), y.unwrap_f32()),
            Nx::N64 => op.n64(x.unwrap_f64(), y.unwrap_f64()),
        };
        self.push_value(Val::I32(z as u32))
    }

    #[cfg(feature = "float-types")]
    fn funop(&mut self, n: Nx, op: FUnOp) {
        let x = self.pop_value();
        let z = match n {
            Nx::N32 => Val::F32(op.n32(x.unwrap_f32())),
            Nx::N64 => Val::F64(op.n64(x.unwrap_f64())),
        };
        self.push_value(z);
    }

    #[cfg(feature = "float-types")]
    fn fbinop(&mut self, n: Nx, op: FBinOp) {
        let y = self.pop_value();
        let x = self.pop_value();
        let z = match n {
            Nx::N32 => Val::F32(op.n32(x.unwrap_f32(), y.unwrap_f32())),
            Nx::N64 => Val::F64(op.n64(x.unwrap_f64(), y.unwrap_f64())),
        };
        self.push_value(z);
    }

    fn cvtop(&mut self, op: CvtOp) -> Result<(), Error> {
        #[cfg(feature = "float-types")]
        macro_rules! trunc {
            ($x:expr, $n:tt, $m:tt, $s:tt) => {{
                paste::paste! {
                    let x = [<f $m>]::from_bits($x.[<unwrap_f $m>]());
                    let min = ([<$s $n>]::MIN as [<f $m>]);
                    let max = ([<$s $n>]::MAX as [<f $m>]);
                    let min = if min - 1. == min { min <= x } else { min - 1. < x };
                    (min && x < max + 1.).then(|| Val::[<I $n>](x as [<$s $n>] as [<u $n>]))?
                }
            }};
        }
        #[cfg(feature = "float-types")]
        macro_rules! trunc_sat {
            ($x:expr, $n:tt, $m:tt, $s:tt) => {
                paste::paste! {
                    Val::[<I $n>](match [<f $m>]::from_bits($x.[<unwrap_f $m>]()) {
                        x if x.is_nan() => 0,
                        x if x.is_infinite() && x.is_sign_positive() => [<$s $n>]::MAX,
                        x if x.is_infinite() && x.is_sign_negative() => [<$s $n>]::MIN,
                        x => x as [<$s $n>],
                    } as [<u $n>])
                }
            };
        }
        #[cfg(feature = "float-types")]
        macro_rules! convert {
            ($x:expr, $n:tt, $m:tt, $s:tt) => {
                paste::paste! {
                    Val::[<F $n>](($x.[<unwrap_i $m>]() as [<$s $m>] as [<f $n>]).to_bits())
                }
            };
        }
        #[cfg(feature = "float-types")]
        macro_rules! dispatch {
            ($f:ident, $x:expr, $n:expr, $m:expr, $s:expr) => {
                match ($n, $m, $s) {
                    (Nx::N32, Nx::N32, Sx::U) => $f!($x, 32, 32, u),
                    (Nx::N32, Nx::N32, Sx::S) => $f!($x, 32, 32, i),
                    (Nx::N32, Nx::N64, Sx::U) => $f!($x, 32, 64, u),
                    (Nx::N32, Nx::N64, Sx::S) => $f!($x, 32, 64, i),
                    (Nx::N64, Nx::N32, Sx::U) => $f!($x, 64, 32, u),
                    (Nx::N64, Nx::N32, Sx::S) => $f!($x, 64, 32, i),
                    (Nx::N64, Nx::N64, Sx::U) => $f!($x, 64, 64, u),
                    (Nx::N64, Nx::N64, Sx::S) => $f!($x, 64, 64, i),
                }
            };
        }
        use CvtOp::*;
        let x = self.pop_value();
        let z = try {
            match op {
                Wrap => Val::I32(x.unwrap_i64() as u32),
                Extend(Sx::U) => Val::I64(x.unwrap_i32() as u64),
                Extend(Sx::S) => Val::I64(x.unwrap_i32() as i32 as u64),
                #[cfg(feature = "float-types")]
                Trunc(n, m, s) => dispatch!(trunc, x, n, m, s),
                #[cfg(feature = "float-types")]
                TruncSat(n, m, s) => dispatch!(trunc_sat, x, n, m, s),
                #[cfg(feature = "float-types")]
                Convert(n, m, s) => dispatch!(convert, x, n, m, s),
                #[cfg(feature = "float-types")]
                Demote => Val::F32((f64::from_bits(x.unwrap_f64()) as f32).to_bits()),
                #[cfg(feature = "float-types")]
                Promote => Val::F64((f32::from_bits(x.unwrap_f32()) as f64).to_bits()),
                #[cfg(feature = "float-types")]
                IReinterpret(n) => match n {
                    Nx::N32 => Val::I32(x.unwrap_f32()),
                    Nx::N64 => Val::I64(x.unwrap_f64()),
                },
                #[cfg(feature = "float-types")]
                FReinterpret(n) => match n {
                    Nx::N32 => Val::F32(x.unwrap_i32()),
                    Nx::N64 => Val::F64(x.unwrap_i64()),
                },
            }
        };
        self.push_value_or_trap(z)
    }

    fn extend(&mut self, n: Bx) {
        let x = self.pop_value();
        let z = match n {
            Bx::N32B8 => Val::I32(x.unwrap_i32() as i8 as u32),
            Bx::N32B16 => Val::I32(x.unwrap_i32() as i16 as u32),
            Bx::N64B8 => Val::I64(x.unwrap_i64() as i8 as u64),
            Bx::N64B16 => Val::I64(x.unwrap_i64() as i16 as u64),
            Bx::N64B32 => Val::I64(x.unwrap_i64() as i32 as u64),
        };
        self.push_value(z)
    }

    fn invoke(mut self, store: &mut Store<'m>, ptr: Ptr) -> Result<ThreadResult<'m>, Error> {
        // TODO: This should be based on actual size in RAM.
        const MAX_FRAMES: usize = 1000;
        if self.frames.len() >= MAX_FRAMES {
            return Err(trap());
        }
        let t = store.func_type(ptr);
        let inst_id = match ptr.instance() {
            Side::Host => {
                let index = ptr.index() as usize;
                let t = store.funcs[index].1;
                let arity = t.results.len();
                let args = self.pop_values(t.params.len());
                store.threads.push(Continuation { thread: self, arity, index, args });
                return Ok(ThreadResult::Host);
            }
            Side::Wasm(x) => x,
        };
        let (mut parser, side_table) = store.insts[inst_id].module.func(ptr.index());
        let mut locals = self.pop_values(t.params.len());
        append_locals(&mut parser, &mut locals);
        let ret = self.parser;
        self.parser = parser;
        let prev_stack = self.values().len();
        let frame = Frame::new(inst_id, t.results.len(), ret, locals, side_table, prev_stack);
        self.frames.push(frame);
        Ok(ThreadResult::Continue(self))
    }
}

fn table_init(d: usize, s: usize, n: usize, table: &mut Table, elems: &[Val]) -> Result<(), Error> {
    if s.checked_add(n).is_none_or(|x| x > elems.len())
        || d.checked_add(n).is_none_or(|x| x > table.elems.len())
    {
        Err(trap())
    } else {
        table.elems[d ..][.. n].copy_from_slice(&elems[s ..][.. n]);
        Ok(())
    }
}

fn memory_init(d: usize, s: usize, n: usize, mem: &mut Memory, data: &[u8]) -> Result<(), Error> {
    if s.checked_add(n).is_none_or(|x| x > data.len())
        || d.checked_add(n).is_none_or(|x| x > mem.len() as usize)
    {
        memory_too_small(d, n, mem);
        Err(trap())
    } else {
        mem.data[d ..][.. n].copy_from_slice(&data[s ..][.. n]);
        Ok(())
    }
}

#[derive(Debug)]
struct Frame<'m> {
    inst_id: usize,
    arity: usize,
    ret: Parser<'m>,
    locals: Vec<Val>,
    side_table: Cursor<'m, BranchTableEntry>,
    /// Total length of the value stack in the thread prior to this frame.
    prev_stack: usize,
    // TODO: We should be able to get rid of this by using the side-table.
    labels_cnt: usize,
}

impl<'m> Frame<'m> {
    fn new(
        inst_id: usize, arity: usize, ret: Parser<'m>, locals: Vec<Val>,
        side_table: Cursor<'m, BranchTableEntry>, prev_stack: usize,
    ) -> Self {
        Frame { inst_id, arity, ret, locals, side_table, prev_stack, labels_cnt: 1 }
    }

    fn skip_jump(&mut self) {
        self.side_table.adjust_start(1);
    }

    fn take_jump(&mut self, offset: usize) -> isize {
        self.side_table.adjust_start(offset as isize);
        let entry = self.side_table.get(0).view::<Use>().into_ok();
        self.side_table.adjust_start(entry.delta_stp as isize);
        entry.delta_ip as isize
    }
}

impl Table {
    fn new(type_: TableType) -> Self {
        Table {
            max: type_.limits.max,
            elems: vec![Val::Null(type_.item); type_.limits.min as usize],
        }
    }
}

#[derive(Debug, Default)]
struct Memory<'m> {
    // May be shorter than the maximum length for the module, but not larger.
    data: &'m mut [u8],
    // The size currently available to the module. May be larger than the actual data.
    size: u32,
    max: u32,
}

impl<'m> Memory<'m> {
    fn init(&mut self, mut data: &'m mut [u8], limits: Limits) -> Result<(), Error> {
        if !data.as_ptr().is_aligned_to(MEMORY_ALIGN) {
            return Err(invalid());
        }
        if limits.max < 0x10000 {
            let max = core::cmp::min(limits.max as usize * 0x10000, data.len());
            data = &mut data[.. max];
        }
        self.data = data;
        // TODO: Figure out if we need to do this or whether we can rely on the caller to provide a
        // zeroed-out slice.
        self.data.fill(0);
        self.size = limits.min;
        self.max = limits.max;
        Ok(())
    }

    fn len(&self) -> u32 {
        core::cmp::min(self.data.len() as u32, self.size * 0x10000)
    }
}

impl Global {
    fn new(value: Val) -> Self {
        Global { value }
    }
}

trait Growable {
    type Item: Copy;
    fn size(&self) -> u32;
    fn max(&self) -> u32;
    fn grow(&mut self, n: u32, x: Self::Item);
}

impl Growable for Memory<'_> {
    type Item = ();
    fn size(&self) -> u32 {
        self.size
    }
    fn max(&self) -> u32 {
        self.max
    }
    fn grow(&mut self, n: u32, _: Self::Item) {
        self.size = n;
    }
}

impl Growable for Table {
    type Item = Val;
    fn size(&self) -> u32 {
        self.elems.len() as u32
    }
    fn max(&self) -> u32 {
        self.max
    }
    fn grow(&mut self, n: u32, x: Self::Item) {
        self.elems.resize(n as usize, x);
    }
}

/// Returns the old size or u32::MAX.
fn grow<T: Growable>(elems: &mut T, n: u32, item: T::Item) -> u32 {
    let sz = elems.size();
    let len = match sz.checked_add(n) {
        Some(x) if x <= elems.max() => x,
        _ => return u32::MAX,
    };
    elems.grow(len, item);
    sz
}

#[derive(Debug, Default)]
struct Component<T> {
    ext: Vec<Ptr>,
    int: T,
}

impl<T> Component<T> {
    fn ptr(&self, inst_id: usize, x: u32) -> Ptr {
        match x.checked_sub(self.ext.len() as u32) {
            None => self.ext[x as usize],
            Some(x) => Ptr::new(Side::Wasm(inst_id), x),
        }
    }
}

macro_rules! impl_val_unwrap {
    ($n:ident, $T:ident, $t:ident) => {
        impl Val {
            pub fn $n(self) -> $t {
                match self {
                    Val::$T(x) => x,
                    _ => unreachable!(),
                }
            }
        }
    };
}
impl_val_unwrap!(unwrap_i32, I32, u32);
impl_val_unwrap!(unwrap_i64, I64, u64);
#[cfg(feature = "float-types")]
impl_val_unwrap!(unwrap_f32, F32, u32);
#[cfg(feature = "float-types")]
impl_val_unwrap!(unwrap_f64, F64, u64);
impl_val_unwrap!(unwrap_ref, Ref, Ptr);

impl ValType {
    fn default(self) -> Val {
        match self {
            ValType::I32 => Val::I32(0),
            ValType::I64 => Val::I64(0),
            ValType::F32 => support_if!("float-types"[], Val::F32(0), unreachable!()),
            ValType::F64 => support_if!("float-types"[], Val::F64(0), unreachable!()),
            ValType::V128 => support_if!("vector-types"[], Val::V128(0), unreachable!()),
            ValType::FuncRef => Val::Null(RefType::FuncRef),
            ValType::ExternRef => Val::Null(RefType::ExternRef),
        }
    }

    fn contains(self, value: Val) -> bool {
        match (self, value) {
            (ValType::I32, Val::I32(_))
            | (ValType::I64, Val::I64(_))
            | (ValType::FuncRef, Val::Null(RefType::FuncRef) | Val::Ref(_))
            | (ValType::ExternRef, Val::Null(RefType::ExternRef) | Val::RefExtern(_)) => true,
            #[cfg(feature = "float-types")]
            (ValType::F32, Val::F32(_)) | (ValType::F64, Val::F64(_)) => true,
            #[cfg(feature = "vector-types")]
            (ValType::V128, Val::V128(_)) => true,
            _ => false,
        }
    }
}

fn append_locals(parser: &mut Parser, locals: &mut Vec<Val>) {
    for _ in 0 .. parser.parse_vec().into_ok() {
        let len = parser.parse_u32().into_ok() as usize;
        let val = parser.parse_valtype().into_ok().default();
        locals.extend(core::iter::repeat_n(val, len));
    }
}

fn check_types(types: &[ValType], values: &[Val]) -> Result<(), Error> {
    check(types.len() == values.len())?;
    check(types.iter().zip(values.iter()).all(|(t, x)| t.contains(*x)))
}

fn memory_too_small(x: usize, n: usize, mem: &Memory) {
    #[cfg(not(feature = "debug"))]
    let _ = (x, n, mem);
    #[cfg(feature = "debug")]
    eprintln!("Memory too small: {x} + {n} > {}", mem.len());
}
