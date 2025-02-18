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

use alloc::collections::BTreeSet;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::marker::PhantomData;
use core::ops::Range;

use crate::error::*;
use crate::side_table::*;
use crate::syntax::*;
use crate::toctou::*;
use crate::util::*;
use crate::*;

/// Checks whether a WASM module in binary format is valid, and returns the side table.
pub fn prepare(binary: &[u8]) -> Result<Vec<MetadataEntry>, Error> {
    validate::<Prepare>(binary)
}

pub fn merge(binary: &[u8], side_table: Vec<MetadataEntry>) -> Result<Vec<u8>, Error> {
    let mut wasm = vec![];
    wasm.extend_from_slice(&binary[0 .. 8]);
    wasm.push(0);
    let side_table_slice = serialize(side_table.as_slice())?;
    leb128(side_table_slice.len(), &mut wasm);
    let name = "wasefire-sidetable";
    leb128(name.len(), &mut wasm);
    wasm.extend_from_slice(name.as_bytes());
    wasm.extend_from_slice(side_table_slice.as_slice());
    wasm.extend_from_slice(&binary[8 ..]);
    Ok(wasm)
}

#[allow(dead_code)]
#[allow(unused_variables)]
/// Checks whether a WASM module with the side table in binary format is valid.
pub fn verify(binary: &[u8]) -> Result<(), Error> {
    validate::<Verify>(binary)
}

fn validate<M: ValidMode>(binary: &[u8]) -> Result<M::Result, Error> {
    Context::<M>::default().check_module(&mut Parser::new(binary))
}

trait ValidMode: Default {
    type Branches<'m>: BranchesApi<'m>;
    type BranchTable<'a, 'm>: BranchTableApi<'m>;
    type SideTable<'m>;
    type Result;

    fn parse_side_table<'m>(parser: &mut Parser<'m>) -> Result<Self::SideTable<'m>, Error>;
    fn next_branch_table<'a, 'm>(
        side_table: &'a mut Self::SideTable<'m>, type_idx: usize, parser_range: Range<usize>,
    ) -> Result<Self::BranchTable<'a, 'm>, Error>;
    fn side_table_result(side_table: Self::SideTable<'_>) -> Self::Result;
}

trait BranchesApi<'m>: Default + IntoIterator<Item = SideTableBranch<'m>> {
    fn push_branch(&mut self, branch: SideTableBranch<'m>) -> CheckResult;
}

trait BranchTableApi<'m> {
    fn stitch_branch(
        &mut self, source: SideTableBranch<'m>, target: SideTableBranch<'m>,
    ) -> CheckResult;
    fn patch_branch(&self, source: SideTableBranch<'m>) -> Result<SideTableBranch<'m>, Error>;
    fn allocate_branch(&mut self);
    fn next_index(&self) -> usize;
}

#[derive(Default)]
struct Prepare;
impl ValidMode for Prepare {
    /// List of source branches.
    type Branches<'m> = Vec<SideTableBranch<'m>>;
    type BranchTable<'a, 'm> = &'a mut Vec<BranchTableEntry>;
    type SideTable<'m> = Vec<MetadataEntry>;
    // TODO(dev/fast-interp): Change it to Vec<u8>.
    type Result = Vec<MetadataEntry>;

    fn parse_side_table<'m>(_: &mut Parser<'m>) -> Result<Self::SideTable<'m>, Error> {
        Ok(Vec::new())
    }

    fn next_branch_table<'a, 'm>(
        side_tables: &'a mut Self::SideTable<'m>, type_idx: usize, parser_range: Range<usize>,
    ) -> Result<Self::BranchTable<'a, 'm>, Error> {
        side_tables.push(MetadataEntry { type_idx, parser_range, branch_table: vec![] });
        Ok(&mut side_tables.last_mut().unwrap().branch_table)
    }

    fn side_table_result(side_table: Self::SideTable<'_>) -> Self::Result {
        side_table
    }
}

impl<'m> BranchesApi<'m> for Vec<SideTableBranch<'m>> {
    fn push_branch(&mut self, branch: SideTableBranch<'m>) -> CheckResult {
        Ok(self.push(branch))
    }
}

impl<'m> BranchTableApi<'m> for &mut Vec<BranchTableEntry> {
    /// Updates the branch table for source according to target
    fn stitch_branch(
        &mut self, source: SideTableBranch<'m>, target: SideTableBranch<'m>,
    ) -> CheckResult {
        let delta_ip = delta(source, target, |x| x.parser.as_ptr() as isize)?;
        let delta_stp = delta(source, target, |x| x.branch_table as isize)?;
        let val_cnt = u32::try_from(target.result).map_err(|_| {
            #[cfg(feature = "debug")]
            eprintln!("side-table val_cnt overflow {0}", target.result);
            unsupported(if_debug!(Unsupported::SideTable))
        })?;
        let pop_cnt = pop_cnt(source, target)?;
        debug_assert!(self[source.branch_table].is_invalid());
        self[source.branch_table] =
            BranchTableEntry::new(BranchTableEntryView { delta_ip, delta_stp, val_cnt, pop_cnt })?;
        Ok(())
    }

    fn patch_branch(&self, source: SideTableBranch<'m>) -> Result<SideTableBranch<'m>, Error> {
        Ok(source)
    }

    fn allocate_branch(&mut self) {
        self.push(BranchTableEntry::invalid());
    }

    fn next_index(&self) -> usize {
        self.len()
    }
}

struct MetadataView<'m> {
    metadata: Metadata<'m>,
    branch_idx: usize,
}

#[derive(Default)]
struct Verify;
impl ValidMode for Verify {
    /// Contains at most one _target_ branch. Source branches are eagerly patched to
    /// their target branch using the branch table.
    type Branches<'m> = Option<SideTableBranch<'m>>;
    type BranchTable<'a, 'm> = MetadataView<'m>;
    type SideTable<'m> = SideTableView<'m>;
    type Result = ();

    fn parse_side_table<'m>(parser: &mut Parser<'m>) -> Result<Self::SideTable<'m>, Error> {
        check(parser.parse_section_id()? == SectionId::Custom)?;
        let mut section = parser.split_section()?;
        check(section.parse_name()? == "wasefire-sidetable")?;
        SideTableView::new(parser.save())
    }

    fn next_branch_table<'a, 'm>(
        side_table: &'a mut Self::SideTable<'m>, type_idx: usize, parser_range: Range<usize>,
    ) -> Result<Self::BranchTable<'a, 'm>, Error> {
        let metadata = side_table.metadata(side_table.func_idx);
        side_table.func_idx += 1;
        check(metadata.type_idx() == type_idx)?;
        check(metadata.parser_range() == parser_range)?;
        Ok(MetadataView { metadata, branch_idx: 0 })
    }

    fn side_table_result(side_table: Self::SideTable<'_>) -> Self::Result {
        check(side_table.func_idx == side_table.indices.len()).unwrap()
    }
}

impl<'m> BranchesApi<'m> for Option<SideTableBranch<'m>> {
    fn push_branch(&mut self, branch: SideTableBranch<'m>) -> CheckResult {
        check(self.replace(branch).is_none_or(|x| x == branch))
    }
}

impl<'m> BranchTableApi<'m> for MetadataView<'m> {
    fn stitch_branch(
        &mut self, source: SideTableBranch<'m>, target: SideTableBranch<'m>,
    ) -> CheckResult {
        check(source == target)
    }

    fn patch_branch(&self, mut source: SideTableBranch<'m>) -> Result<SideTableBranch<'m>, Error> {
        source.branch_table = self.branch_idx;
        let entry = self.metadata.branch_table()[source.branch_table].view();
        offset_front(source.parser, entry.delta_ip as isize);
        source.branch_table += entry.delta_stp as usize;
        source.result = entry.val_cnt as usize;
        source.stack -= entry.pop_cnt as usize;
        Ok(source)
    }

    fn allocate_branch(&mut self) {
        self.branch_idx += 1;
    }

    fn next_index(&self) -> usize {
        self.branch_idx
    }
}

pub type Parser<'m> = parser::Parser<'m, Check>;
type CheckResult = MResult<(), Check>;

#[derive(Default)]
struct Context<'m, M: ValidMode> {
    types: Vec<FuncType<'m>>,
    funcs: Vec<TypeIdx>,
    tables: Vec<TableType>,
    mems: Vec<MemType>,
    globals: Vec<GlobalType>,
    elems: Vec<RefType>,
    datas: Option<usize>,
    mode: PhantomData<M>,
}

impl<'m, M: ValidMode> Context<'m, M> {
    fn check_module(&mut self, parser: &mut Parser<'m>) -> Result<M::Result, Error> {
        check(parser.parse_bytes(8)? == b"\0asm\x01\0\0\0")?;
        let mut side_table = M::parse_side_table(parser)?;
        let module_start = parser.save().as_ptr() as usize;
        if let Some(mut parser) = self.check_section(parser, SectionId::Type)? {
            let n = parser.parse_vec()?;
            self.types.reserve(n);
            for _ in 0 .. n {
                self.types.push(parser.parse_functype()?);
            }
            check(parser.is_empty())?;
        }
        if let Some(mut parser) = self.check_section(parser, SectionId::Import)? {
            for _ in 0 .. parser.parse_vec()? {
                self.add_import(&mut parser)?;
            }
            check(parser.is_empty())?;
        }
        let imported_funcs = self.funcs.len();
        if let Some(mut parser) = self.check_section(parser, SectionId::Function)? {
            for _ in 0 .. parser.parse_vec()? {
                self.add_functype(parser.parse_typeidx()?)?;
            }
            check(parser.is_empty())?;
        }
        let mut refs = vec![false; self.funcs.len()];
        if let Some(mut parser) = self.check_section(parser, SectionId::Table)? {
            for _ in 0 .. parser.parse_vec()? {
                self.add_tabletype(parser.parse_tabletype()?)?;
            }
            check(parser.is_empty())?;
        }
        if let Some(mut parser) = self.check_section(parser, SectionId::Memory)? {
            for _ in 0 .. parser.parse_vec()? {
                self.add_memtype(parser.parse_memtype()?)?;
            }
            check(parser.is_empty())?;
        }
        check(self.mems.len() <= 1)?;
        let globals_len = self.globals.len();
        if let Some(mut parser) = self.check_section(parser, SectionId::Global)? {
            for _ in 0 .. parser.parse_vec()? {
                self.add_globaltype(parser.parse_globaltype()?)?;
                Expr::check_const(
                    self,
                    &mut parser,
                    &mut refs,
                    globals_len,
                    self.globals.last().unwrap().value.into(),
                )?;
            }
            check(parser.is_empty())?;
        }
        if let Some(mut parser) = self.check_section(parser, SectionId::Export)? {
            let mut names = BTreeSet::new();
            for _ in 0 .. parser.parse_vec()? {
                let name = parser.parse_name()?;
                check(names.insert(name))?;
                let desc = parser.parse_exportdesc()?;
                self.check_exportdesc(&desc)?;
                if let ExportDesc::Func(x) = desc {
                    refs[x as usize] = true;
                }
            }
            check(parser.is_empty())?;
        }
        if let Some(mut parser) = self.check_section(parser, SectionId::Start)? {
            let x = parser.parse_funcidx()?;
            let t = self.functype(x)?;
            check(t.params.is_empty() && t.results.is_empty())?;
            check(parser.is_empty())?;
        }
        if let Some(mut parser) = self.check_section(parser, SectionId::Element)? {
            for _ in 0 .. parser.parse_vec()? {
                parser.parse_elem(&mut ParseElem::new(self, &mut refs, globals_len))?;
            }
            check(parser.is_empty())?;
        }
        if let Some(mut parser) = self.check_section(parser, SectionId::DataCount)? {
            self.datas = Some(parser.parse_u32()? as usize);
            check(parser.is_empty())?;
        }
        if let Some(mut parser) = self.check_section(parser, SectionId::Code)? {
            check(self.funcs.len() == imported_funcs + parser.parse_vec()?)?;
            for x in imported_funcs .. self.funcs.len() {
                let size = parser.parse_u32()? as usize;
                let mut parser = parser.split_at(size)?;
                let parser_start = parser.save().as_ptr() as usize - module_start;
                let t = self.functype(x as FuncIdx).unwrap();
                let mut locals = t.params.to_vec();
                parser.parse_locals(&mut locals)?;
                let parser_range = Range { start: parser_start, end: parser_start + size };
                let branch_table =
                    M::next_branch_table(&mut side_table, self.funcs[x] as usize, parser_range)?;
                Expr::check_body(self, &mut parser, &refs, locals, t.results, branch_table)?;
                check(parser.is_empty())?;
            }
            check(parser.is_empty())?;
        } else {
            check(self.funcs.len() == imported_funcs)?;
        }
        if let Some(mut parser) = self.check_section(parser, SectionId::Data)? {
            let n = parser.parse_vec()?;
            check(self.datas.is_none_or(|m| m == n))?;
            for _ in 0 .. n {
                parser.parse_data(&mut ParseData::new(self, &mut refs, globals_len))?;
            }
            check(parser.is_empty())?;
        } else {
            check(self.datas.is_none_or(|m| m == 0))?;
        }
        self.check_section(parser, SectionId::Custom)?;
        check(parser.is_empty())?;
        Ok(M::side_table_result(side_table))
    }

    fn check_section(
        &mut self, parser: &mut Parser<'m>, expected_id: SectionId,
    ) -> Result<Option<Parser<'m>>, Error> {
        let (actual_id, next_parser) = loop {
            if parser.is_empty() {
                return Ok(None);
            }
            let mut next_parser = parser.clone();
            let id = next_parser.parse_section_id()?;
            if id != SectionId::Custom {
                break (id, next_parser);
            }
            *parser = next_parser;
            let mut section = parser.split_section()?;
            section.parse_name()?;
            // We ignore the remaining bytes.
        };
        match expected_id.order().cmp(&actual_id.order()) {
            Ordering::Less => Ok(None),
            Ordering::Equal => {
                *parser = next_parser;
                Ok(Some(parser.split_section()?))
            }
            Ordering::Greater => Err(invalid()),
        }
    }

    fn add_import(&mut self, parser: &mut Parser) -> CheckResult {
        parser.parse_name()?;
        parser.parse_name()?;
        match parser.parse_importdesc()? {
            ImportDesc::Func(x) => self.add_functype(x),
            ImportDesc::Table(t) => self.add_tabletype(t),
            ImportDesc::Mem(m) => self.add_memtype(m),
            ImportDesc::Global(g) => self.add_globaltype(g),
        }
    }

    fn add_functype(&mut self, x: TypeIdx) -> CheckResult {
        check((x as usize) < self.types.len())?;
        self.funcs.push(x);
        Ok(())
    }

    fn add_tabletype(&mut self, t: TableType) -> CheckResult {
        check(t.limits.valid(TABLE_MAX))?;
        self.tables.push(t);
        Ok(())
    }

    fn add_memtype(&mut self, m: MemType) -> CheckResult {
        check(m.valid(MEM_MAX))?;
        self.mems.push(m);
        Ok(())
    }

    fn add_globaltype(&mut self, g: GlobalType) -> CheckResult {
        self.globals.push(g);
        Ok(())
    }

    fn check_exportdesc(&self, desc: &ExportDesc) -> CheckResult {
        let (&x, n) = match desc {
            ExportDesc::Func(x) => (x, self.funcs.len()),
            ExportDesc::Table(x) => (x, self.tables.len()),
            ExportDesc::Mem(x) => (x, self.mems.len()),
            ExportDesc::Global(x) => (x, self.globals.len()),
        };
        check((x as usize) < n)
    }

    fn type_(&self, x: TypeIdx) -> Result<FuncType<'m>, Error> {
        self.types.get(x as usize).cloned().ok_or_else(invalid)
    }

    fn functype(&self, x: FuncIdx) -> Result<FuncType<'m>, Error> {
        self.type_(*self.funcs.get(x as usize).ok_or_else(invalid)?)
    }

    fn table(&self, x: TableIdx) -> Result<&TableType, Error> {
        self.tables.get(x as usize).ok_or_else(invalid)
    }

    fn mem(&self, x: MemIdx) -> Result<&MemType, Error> {
        self.mems.get(x as usize).ok_or_else(invalid)
    }

    fn global(&self, x: GlobalIdx) -> Result<&GlobalType, Error> {
        self.globals.get(x as usize).ok_or_else(invalid)
    }

    fn elem(&self, x: ElemIdx) -> Result<RefType, Error> {
        self.elems.get(x as usize).cloned().ok_or_else(invalid)
    }

    fn data(&self, x: DataIdx) -> CheckResult {
        check(self.datas.is_some_and(|n| (x as usize) < n))
    }
}

struct ParseElem<'a, 'm, M: ValidMode> {
    context: &'a mut Context<'m, M>,
    refs: &'a mut [bool],
    num_global_imports: usize,
    table_type: OpdType,
    elem_type: RefType,
}

impl<'a, 'm, M: ValidMode> ParseElem<'a, 'm, M> {
    fn new(
        context: &'a mut Context<'m, M>, refs: &'a mut [bool], num_global_imports: usize,
    ) -> Self {
        Self {
            context,
            num_global_imports,
            refs,
            table_type: OpdType::Bottom,
            elem_type: RefType::FuncRef,
        }
    }
}

impl<'m, M: ValidMode> parser::ParseElem<'m, Check> for ParseElem<'_, 'm, M> {
    fn mode(&mut self, mode: parser::ElemMode<'_, 'm, Check>) -> MResult<(), Check> {
        if let parser::ElemMode::Active { table, offset } = mode {
            Expr::check_const(
                self.context,
                offset,
                self.refs,
                self.num_global_imports,
                ValType::I32.into(),
            )?;
            self.table_type = self.context.table(table)?.item.into();
        }
        Ok(())
    }

    fn type_(&mut self, type_: RefType) -> MResult<(), Check> {
        self.elem_type = type_;
        check(self.table_type.matches(type_.into()))?;
        self.context.elems.push(type_);
        Ok(())
    }

    fn init_funcidx(&mut self, x: FuncIdx) -> MResult<(), Check> {
        check((x as usize) < self.context.funcs.len())?;
        self.refs[x as usize] = true;
        Ok(())
    }

    fn init_expr(&mut self, parser: &mut parser::Parser<'m, Check>) -> MResult<(), Check> {
        Expr::check_const(
            self.context,
            parser,
            self.refs,
            self.num_global_imports,
            ValType::from(self.elem_type).into(),
        )
    }
}

struct ParseData<'a, 'm, M: ValidMode> {
    context: &'a mut Context<'m, M>,
    refs: &'a mut [bool],
    num_global_imports: usize,
}

impl<'a, 'm, M: ValidMode> ParseData<'a, 'm, M> {
    fn new(
        context: &'a mut Context<'m, M>, refs: &'a mut [bool], num_global_imports: usize,
    ) -> Self {
        Self { context, refs, num_global_imports }
    }
}

impl<'m, M: ValidMode> parser::ParseData<'m, Check> for ParseData<'_, 'm, M> {
    fn mode(&mut self, mode: parser::DataMode<'_, 'm, Check>) -> MResult<(), Check> {
        if let parser::DataMode::Active { memory, offset } = mode {
            self.context.mem(memory)?;
            Expr::check_const(
                self.context,
                offset,
                self.refs,
                self.num_global_imports,
                ValType::I32.into(),
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
enum OpdType {
    Bottom = 0x80,
    I32 = 0x7f,
    I64 = 0x7e,
    F32 = 0x7d,
    F64 = 0x7c,
    V128 = 0x7b,
    FuncRef = 0x70,
    ExternRef = 0x6f,
}

macro_rules! impl_type_conv {
    ($from:ident => $to:ident [$($val:ident)*]) => {
        impl From<$from> for $to {
            fn from(x: $from) -> $to {
                match x {
                    $($from::$val => $to::$val),*
                }
            }
        }
    };
}
impl_type_conv!(NumType => ValType [I32 I64 F32 F64]);
impl_type_conv!(RefType => ValType [FuncRef ExternRef]);
impl_type_conv!(NumType => OpdType [I32 I64 F32 F64]);
impl_type_conv!(RefType => OpdType [FuncRef ExternRef]);
impl_type_conv!(ValType => OpdType [I32 I64 F32 F64 V128 FuncRef ExternRef]);

impl OpdType {
    fn matches(self, other: ValType) -> bool {
        self == OpdType::Bottom || self == other.into()
    }

    fn unify(self, other: OpdType) -> Result<OpdType, Error> {
        match (self, other) {
            (OpdType::Bottom, x) | (x, OpdType::Bottom) => Ok(x),
            (x, y) if x == y => Ok(x),
            _ => Err(invalid()),
        }
    }

    fn is_num(self) -> bool {
        matches!(self, OpdType::Bottom | OpdType::I32 | OpdType::I64 | OpdType::F32 | OpdType::F64)
    }

    fn is_vec(self) -> bool {
        matches!(self, OpdType::Bottom | OpdType::V128)
    }

    fn is_ref(self) -> bool {
        matches!(self, OpdType::Bottom | OpdType::FuncRef | OpdType::ExternRef)
    }
}

struct Expr<'a, 'm, M: ValidMode> {
    context: &'a Context<'m, M>,
    parser: &'a mut Parser<'m>,
    globals_len: usize,
    /// Whether the expression is const and the function references.
    is_const: Result<&'a mut [bool], &'a [bool]>,
    is_body: bool,
    locals: Vec<ValType>,
    labels: Vec<Label<'m, M>>,
    branch_table: Option<M::BranchTable<'a, 'm>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct SideTableBranch<'m> {
    parser: &'m [u8],
    branch_table: usize,
    stack: usize,
    result: usize, // unused (zero) for source branches
}

#[derive(Debug, Default)]
struct Label<'m, M: ValidMode> {
    type_: FuncType<'m>,
    /// Whether an `else` is possible before `end`.
    kind: LabelKind<'m>,
    /// Whether the bottom of the stack is polymorphic.
    polymorphic: bool,
    stack: Vec<OpdType>,
    branches: M::Branches<'m>,
    /// Total stack length of the labels in this function up to this label.
    prev_stack: usize,
}

#[derive(Debug, Default, Clone)]
enum LabelKind<'m> {
    #[default]
    Block,
    Loop(SideTableBranch<'m>),
    If(SideTableBranch<'m>),
}

impl<'a, 'm, M: ValidMode> Expr<'a, 'm, M> {
    fn new(
        context: &'a Context<'m, M>, parser: &'a mut Parser<'m>,
        is_const: Result<&'a mut [bool], &'a [bool]>, branch_table: Option<M::BranchTable<'a, 'm>>,
    ) -> Self {
        Self {
            context,
            parser,
            globals_len: context.globals.len(),
            is_const,
            is_body: false,
            locals: vec![],
            labels: vec![Label::default()],
            branch_table,
        }
    }

    fn check_const(
        context: &'a Context<'m, M>, parser: &'a mut Parser<'m>, refs: &'a mut [bool],
        num_global_imports: usize, expected: ResultType<'m>,
    ) -> CheckResult {
        let mut expr = Expr::new(context, parser, Ok(refs), None);
        expr.globals_len = num_global_imports;
        expr.label().type_.results = expected;
        expr.check()
    }

    fn check_body(
        context: &'a Context<'m, M>, parser: &'a mut Parser<'m>, refs: &'a [bool],
        locals: Vec<ValType>, results: ResultType<'m>, branch_table: M::BranchTable<'a, 'm>,
    ) -> CheckResult {
        let mut expr = Expr::new(context, parser, Err(refs), Some(branch_table));
        expr.is_body = true;
        expr.locals = locals;
        expr.label().type_.results = results;
        expr.check()
    }

    fn check(&mut self) -> CheckResult {
        while !self.labels.is_empty() {
            self.instr()?;
        }
        Ok(())
    }

    fn instr(&mut self) -> CheckResult {
        use Instr::*;
        let saved = self.parser.save();
        let instr = self.parser.parse_instr()?;
        if matches!(instr, End) {
            return self.end_label();
        }
        if self.is_const.is_ok() {
            match instr {
                GlobalGet(x) => {
                    let x = x as usize;
                    let n = self.globals_len;
                    check(x < n && self.context.globals[x].mutable == Mut::Const)?;
                }
                I32Const(_) => (),
                I64Const(_) => (),
                #[cfg(feature = "float-types")]
                F32Const(_) => (),
                #[cfg(feature = "float-types")]
                F64Const(_) => (),
                RefNull(_) => (),
                RefFunc(_) => (),
                _ => return Err(invalid()),
            }
        }
        match instr {
            Unreachable => self.stack_polymorphic(),
            Nop => (),
            Block(b) => self.push_label(self.blocktype(&b)?, LabelKind::Block)?,
            Loop(b) => {
                let type_ = self.blocktype(&b)?;
                let mut target = self.branch_target(type_.params.len());
                target.parser = saved;
                self.push_label(type_, LabelKind::Loop(target))?
            }
            If(b) => {
                self.pop_check(ValType::I32)?;
                let branch = self.branch_source();
                self.push_label(self.blocktype(&b)?, LabelKind::If(branch))?;
            }
            Else => {
                match core::mem::replace(&mut self.label().kind, LabelKind::Block) {
                    LabelKind::If(source) => {
                        let result = self.label().type_.results.len();
                        let mut target = self.branch_target(result);
                        target.branch_table += 1;
                        self.branch_table.as_mut().unwrap().stitch_branch(source, target)?;
                    }
                    _ => Err(invalid())?,
                }
                let FuncType { params, results } = self.label().type_;
                self.pops(results)?;
                check(self.stack().is_empty())?;
                self.label().polymorphic = false;
                self.pushs(params);
                self.br_label(0)?;
            }
            End => unreachable!(),
            Br(l) => {
                let res = self.br_label(l)?;
                self.pops(res)?;
                self.stack_polymorphic();
            }
            BrIf(l) => {
                self.pop_check(ValType::I32)?;
                let res = self.br_label(l)?;
                self.swaps(res)?;
            }
            BrTable(ls, ln) => {
                self.pop_check(ValType::I32)?;
                let tn = self.br_label(ln)?;
                self.peeks(tn)?;
                for l in ls {
                    let t = self.br_label(l)?;
                    check(tn.len() == t.len())?;
                    self.peeks(t)?;
                }
                self.stack_polymorphic();
            }
            Return => {
                check(self.is_body)?;
                self.pops(self.labels[0].type_.results)?;
                self.stack_polymorphic();
            }
            Call(x) => self.call(self.context.functype(x)?)?,
            CallIndirect(x, y) => {
                check(self.context.table(x)?.item == RefType::FuncRef)?;
                self.pop_check(ValType::I32)?;
                self.call(self.context.type_(y)?)?;
            }
            Drop => drop(self.pop()?),
            Select(None) => {
                self.pop_check(ValType::I32)?;
                let t = self.pop()?.unify(self.pop()?)?;
                check(t.is_num() || t.is_vec())?;
                self.push(t);
            }
            Select(Some(t)) => {
                check(t.len() == 1)?;
                self.pops([t[0], t[0], ValType::I32][..].into())?;
                self.pushs(t[0].into());
            }
            LocalGet(x) => self.push(self.local(x)?.into()),
            LocalSet(x) => self.pop_check(self.local(x)?)?,
            LocalTee(x) => self.swap(self.local(x)?)?,
            GlobalGet(x) => self.push(self.context.global(x)?.value.into()),
            GlobalSet(x) => {
                let g = self.context.global(x)?;
                check(g.mutable == Mut::Var)?;
                self.pop_check(g.value)?;
            }
            TableGet(x) => {
                self.pop_check(ValType::I32)?;
                self.push(self.context.table(x)?.item.into());
            }
            TableSet(x) => {
                self.pops([ValType::I32, self.context.table(x)?.item.into()][..].into())?;
            }
            ILoad(n, m) => self.load(false, NumType::i(n), n.into(), m)?,
            #[cfg(feature = "float-types")]
            FLoad(n, m) => self.load(false, NumType::f(n), n.into(), m)?,
            ILoad_(b, _, m) => self.load(false, NumType::i(b.into()), b.into(), m)?,
            IStore(n, m) => self.store(false, NumType::i(n), n.into(), m)?,
            #[cfg(feature = "float-types")]
            FStore(n, m) => self.store(false, NumType::f(n), n.into(), m)?,
            IStore_(b, m) => self.store(false, NumType::i(b.into()), b.into(), m)?,
            MemorySize => {
                check(!self.context.mems.is_empty())?;
                self.push(OpdType::I32);
            }
            MemoryGrow => {
                check(!self.context.mems.is_empty())?;
                self.swap(ValType::I32)?;
            }
            I32Const(_) => self.push(OpdType::I32),
            I64Const(_) => self.push(OpdType::I64),
            #[cfg(feature = "float-types")]
            F32Const(_) => self.push(OpdType::F32),
            #[cfg(feature = "float-types")]
            F64Const(_) => self.push(OpdType::F64),
            ITestOp(n, _) => self.testop(NumType::i(n))?,
            IRelOp(n, _) => self.relop(NumType::i(n))?,
            #[cfg(feature = "float-types")]
            FRelOp(n, _) => self.relop(NumType::f(n))?,
            IUnOp(n, _) => self.unop(NumType::i(n))?,
            #[cfg(feature = "float-types")]
            FUnOp(n, _) => self.unop(NumType::f(n))?,
            IBinOp(n, _) => self.binop(NumType::i(n))?,
            #[cfg(feature = "float-types")]
            FBinOp(n, _) => self.binop(NumType::f(n))?,
            CvtOp(op) => self.cvtop(op.dst(), op.src())?,
            IExtend(b) => self.unop(NumType::i(b.into()))?,
            RefNull(t) => self.push(t.into()),
            RefIsNull => {
                check(self.pop()?.is_ref())?;
                self.push(OpdType::I32);
            }
            RefFunc(x) => {
                check((x as usize) < self.context.funcs.len())?;
                match &mut self.is_const {
                    Ok(refs) => refs[x as usize] = true,
                    Err(refs) => check(refs[x as usize])?,
                }
                self.push(OpdType::FuncRef);
            }
            MemoryInit(x) => {
                check(!self.context.mems.is_empty())?;
                self.context.data(x)?;
                self.pops([ValType::I32; 3][..].into())?;
            }
            DataDrop(x) => self.context.data(x)?,
            MemoryCopy | MemoryFill => {
                check(!self.context.mems.is_empty())?;
                self.pops([ValType::I32; 3][..].into())?;
            }
            TableInit(x, y) => {
                check(self.context.table(x)?.item == self.context.elem(y)?)?;
                self.pops([ValType::I32; 3][..].into())?;
            }
            ElemDrop(x) => drop(self.context.elem(x)?),
            TableCopy(x, y) => {
                check(self.context.table(x)?.item == self.context.table(y)?.item)?;
                self.pops([ValType::I32; 3][..].into())?;
            }
            TableGrow(x) => {
                let t = self.context.table(x)?.item;
                self.pops([t.into(), ValType::I32][..].into())?;
                self.push(OpdType::I32);
            }
            TableSize(x) => {
                self.context.table(x)?;
                self.push(OpdType::I32);
            }
            TableFill(x) => {
                let t = self.context.table(x)?.item;
                self.pops([ValType::I32, t.into(), ValType::I32][..].into())?;
            }
        }
        Ok(())
    }

    fn blocktype(&self, b: &BlockType) -> Result<FuncType<'m>, Error> {
        Ok(match *b {
            BlockType::None => FuncType { params: ().into(), results: ().into() },
            BlockType::Type(t) => FuncType { params: ().into(), results: t.into() },
            BlockType::Index(x) => self.context.type_(x)?,
        })
    }

    fn local(&self, x: LocalIdx) -> Result<ValType, Error> {
        self.locals.get(x as usize).cloned().ok_or_else(invalid)
    }

    fn label(&mut self) -> &mut Label<'m, M> {
        self.labels.last_mut().unwrap()
    }

    fn immutable_label(&self) -> &Label<'m, M> {
        self.labels.last().unwrap()
    }

    fn stack(&mut self) -> &mut Vec<OpdType> {
        &mut self.label().stack
    }

    fn push(&mut self, t: OpdType) {
        self.stack().push(t);
    }

    fn pushs(&mut self, values: ResultType) {
        self.stack().extend(values.iter().cloned().map(OpdType::from));
    }

    fn pop(&mut self) -> Result<OpdType, Error> {
        let label = self.label();
        Ok(match label.stack.pop() {
            Some(x) => x,
            None => {
                check(label.polymorphic)?;
                OpdType::Bottom
            }
        })
    }

    fn pop_check(&mut self, expected: ValType) -> CheckResult {
        check(self.pop()?.matches(expected))
    }

    fn pops(&mut self, expected: ResultType) -> CheckResult {
        for &y in expected.iter().rev() {
            check(self.pop()?.matches(y))?;
        }
        Ok(())
    }

    fn swap(&mut self, t: ValType) -> CheckResult {
        self.pop_check(t)?;
        self.push(t.into());
        Ok(())
    }

    fn for_each(
        &mut self, expected: ResultType, mut f: impl FnMut(&mut OpdType, ValType) -> CheckResult,
    ) -> CheckResult {
        let label = self.label();
        if label.stack.len() < expected.len() {
            check(label.polymorphic)?;
            let k = expected.len() - label.stack.len();
            label.stack.resize(expected.len(), OpdType::Bottom);
            label.stack.rotate_right(k);
        }
        let n = label.stack.len() - expected.len();
        for (x, &y) in label.stack[n ..].iter_mut().zip(expected.iter()) {
            f(x, y)?;
        }
        Ok(())
    }

    fn swaps(&mut self, expected: ResultType) -> CheckResult {
        #[allow(clippy::unit_arg)]
        self.for_each(expected, |x, y| Ok(*x = x.unify(y.into())?))
    }

    fn peeks(&mut self, expected: ResultType) -> CheckResult {
        self.for_each(expected, |x, y| check(x.matches(y)))
    }

    fn push_label(&mut self, type_: FuncType<'m>, kind: LabelKind<'m>) -> CheckResult {
        self.pops(type_.params)?;
        let stack = type_.params.iter().cloned().map(OpdType::from).collect();
        let prev_label = self.immutable_label();
        let prev_stack = prev_label.prev_stack + prev_label.stack.len();
        let label = Label {
            type_,
            kind,
            polymorphic: false,
            stack,
            branches: Default::default(),
            prev_stack,
        };
        self.labels.push(label);
        Ok(())
    }

    fn end_label(&mut self) -> CheckResult {
        let branches = core::mem::take(&mut self.label().branches);
        if self.is_const.is_ok() {
            assert_eq!(branches.into_iter().count(), 0);
            assert!(matches!(self.label().kind, LabelKind::Block));
        } else {
            let results_len = self.label().type_.results.len();
            let mut target = self.branch_target(results_len);
            for source in branches {
                self.branch_table.as_mut().unwrap().stitch_branch(source, target)?;
            }
            let label = self.label();
            if let LabelKind::If(source) = label.kind {
                check(label.type_.params == label.type_.results)?;
                // SAFETY: This function is only called after parsing an End instruction.
                target.parser = offset_front(target.parser, -1);
                self.branch_table.as_mut().unwrap().stitch_branch(source, target)?;
            }
        }
        let results = self.label().type_.results;
        self.pops(results)?;
        check(self.labels.pop().unwrap().stack.is_empty())?;
        if !self.labels.is_empty() {
            self.pushs(results);
        }
        Ok(())
    }

    fn br_label(&mut self, l: LabelIdx) -> Result<ResultType<'m>, Error> {
        let l = l as usize;
        let n = self.labels.len();
        check(l < n)?;
        let source = self.branch_source();
        let source = self.branch_table.as_ref().unwrap().patch_branch(source)?;
        let label = &mut self.labels[n - l - 1];
        Ok(match label.kind {
            LabelKind::Block | LabelKind::If(_) => {
                label.branches.push_branch(source)?;
                label.type_.results
            }
            LabelKind::Loop(target) => {
                self.branch_table.as_mut().unwrap().stitch_branch(source, target)?;
                label.type_.params
            }
        })
    }

    fn branch_source(&mut self) -> SideTableBranch<'m> {
        let mut branch = self.branch();
        branch.stack += self.stack().len();
        self.branch_table.as_mut().unwrap().allocate_branch();
        branch
    }

    fn branch_target(&self, result: usize) -> SideTableBranch<'m> {
        let mut branch = self.branch();
        branch.result = result;
        branch.stack += result;
        branch
    }

    fn branch(&self) -> SideTableBranch<'m> {
        SideTableBranch {
            parser: self.parser.save(),
            branch_table: self.branch_table.as_ref().unwrap().next_index(),
            stack: self.immutable_label().prev_stack,
            result: 0,
        }
    }

    fn call(&mut self, t: FuncType) -> CheckResult {
        self.pops(t.params)?;
        self.pushs(t.results);
        Ok(())
    }

    fn stack_polymorphic(&mut self) {
        let label = self.label();
        label.stack.clear();
        label.polymorphic = true;
    }

    fn load(&mut self, aligned: bool, t: NumType, n: usize, m: MemArg) -> CheckResult {
        self.check_mem(aligned, n, m)?;
        self.pop_check(ValType::I32)?;
        self.push(t.into());
        Ok(())
    }

    fn store(&mut self, aligned: bool, t: NumType, n: usize, m: MemArg) -> CheckResult {
        self.check_mem(aligned, n, m)?;
        self.pop_check(t.into())?;
        self.pop_check(ValType::I32)?;
        Ok(())
    }

    fn testop(&mut self, t: NumType) -> CheckResult {
        self.pop_check(t.into())?;
        self.push(OpdType::I32);
        Ok(())
    }

    fn relop(&mut self, t: NumType) -> CheckResult {
        self.pops([t.into(); 2][..].into())?;
        self.push(OpdType::I32);
        Ok(())
    }

    fn unop(&mut self, t: NumType) -> CheckResult {
        self.swap(ValType::from(t))
    }

    fn binop(&mut self, t: NumType) -> CheckResult {
        self.pop_check(t.into())?;
        self.swap(ValType::from(t))
    }

    fn cvtop(&mut self, dst: NumType, src: NumType) -> CheckResult {
        self.pop_check(src.into())?;
        self.push(dst.into());
        Ok(())
    }

    fn check_mem(&self, aligned: bool, n: usize, m: MemArg) -> CheckResult {
        check(!self.context.mems.is_empty())?;
        match (aligned, 1usize.checked_shl(m.align), n / 8) {
            (false, Some(m), n) if m <= n => Ok(()),
            (true, Some(m), n) if m == n => Ok(()),
            _ => Err(invalid()),
        }
    }
}

fn delta(
    source: SideTableBranch, target: SideTableBranch, field: fn(SideTableBranch) -> isize,
) -> MResult<i32, Check> {
    let source = field(source);
    let target = field(target);
    let Some(delta) = target.checked_sub(source) else {
        #[cfg(feature = "debug")]
        eprintln!("side-table subtraction overflow {target} - {source}");
        return Err(unsupported(if_debug!(Unsupported::SideTable)));
    };
    i32::try_from(delta).map_err(|_| {
        #[cfg(feature = "debug")]
        eprintln!("side-table conversion overflow {delta}");
        unsupported(if_debug!(Unsupported::SideTable))
    })
}

fn pop_cnt(source: SideTableBranch, target: SideTableBranch) -> MResult<u32, Check> {
    let source = source.stack;
    let target_without_result = target.stack - target.result;
    let Some(delta) = source.checked_sub(target_without_result) else {
        #[cfg(feature = "debug")]
        eprintln!("side-table negative stack delta {source} - {target_without_result}");
        return Err(unsupported(if_debug!(Unsupported::SideTable)));
    };
    if delta < target.result {
        return Ok(0);
    }
    u32::try_from(delta - target.result).map_err(|_| {
        #[cfg(feature = "debug")]
        eprintln!("side-table pop_cnt overflow {delta}");
        unsupported(if_debug!(Unsupported::SideTable))
    })
}
