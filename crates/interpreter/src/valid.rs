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

use crate::error::*;
use crate::syntax::*;
use crate::toctou::*;
use crate::*;

/// Checks whether a WASM module in binary format is valid.
pub fn validate(binary: &[u8]) -> Result<(), Error> {
    Context::default().check_module(&mut Parser::new(binary))
}

type Parser<'m> = parser::Parser<'m, Check>;
type CheckResult = MResult<(), Check>;

#[derive(Default)]
struct Context<'m> {
    types: Vec<FuncType<'m>>,
    funcs: Vec<TypeIdx>,
    tables: Vec<TableType>,
    mems: Vec<MemType>,
    globals: Vec<GlobalType>,
    elems: Vec<RefType>,
    datas: Option<usize>,
}

impl<'m> Context<'m> {
    fn check_module(&mut self, parser: &mut Parser<'m>) -> CheckResult {
        check(parser.parse_bytes(8)? == b"\0asm\x01\0\0\0")?;
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
                let t = self.functype(x as FuncIdx).unwrap();
                let mut locals = t.params.to_vec();
                parser.parse_locals(&mut locals)?;
                Expr::check_body(self, &mut parser, &refs, locals, t.results)?;
                check(parser.is_empty())?;
            }
            check(parser.is_empty())?;
        } else {
            check(self.funcs.len() == imported_funcs)?;
        }
        if let Some(mut parser) = self.check_section(parser, SectionId::Data)? {
            let n = parser.parse_vec()?;
            check(self.datas.map_or(true, |m| m == n))?;
            for _ in 0 .. n {
                parser.parse_data(&mut ParseData::new(self, &mut refs, globals_len))?;
            }
            check(parser.is_empty())?;
        }
        self.check_section(parser, SectionId::Custom)?;
        check(parser.is_empty())
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
        check(self.datas.map_or(false, |n| (x as usize) < n))
    }
}

struct ParseElem<'a, 'm> {
    context: &'a mut Context<'m>,
    refs: &'a mut [bool],
    num_global_imports: usize,
    table_type: OpdType,
    elem_type: RefType,
}

impl<'a, 'm> ParseElem<'a, 'm> {
    fn new(context: &'a mut Context<'m>, refs: &'a mut [bool], num_global_imports: usize) -> Self {
        Self {
            context,
            num_global_imports,
            refs,
            table_type: OpdType::Bottom,
            elem_type: RefType::FuncRef,
        }
    }
}

impl<'a, 'm> parser::ParseElem<'m, Check> for ParseElem<'a, 'm> {
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

struct ParseData<'a, 'm> {
    context: &'a mut Context<'m>,
    refs: &'a mut [bool],
    num_global_imports: usize,
}

impl<'a, 'm> ParseData<'a, 'm> {
    fn new(context: &'a mut Context<'m>, refs: &'a mut [bool], num_global_imports: usize) -> Self {
        Self { context, refs, num_global_imports }
    }
}

impl<'a, 'm> parser::ParseData<'m, Check> for ParseData<'a, 'm> {
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

struct Expr<'a, 'm> {
    context: &'a Context<'m>,
    parser: &'a mut Parser<'m>,
    globals_len: usize,
    /// Whether the expression is const and the function references.
    is_const: Result<&'a mut [bool], &'a [bool]>,
    is_body: bool,
    locals: Vec<ValType>,
    labels: Vec<Label<'m>>,
}

#[derive(Debug, Default)]
struct Label<'m> {
    type_: FuncType<'m>,
    /// Whether an `else` is possible before `end`.
    kind: LabelKind,
    /// Whether the bottom of the stack is polymorphic.
    polymorphic: bool,
    stack: Vec<OpdType>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
enum LabelKind {
    #[default]
    Block,
    Loop,
    If,
}

impl<'a, 'm> Expr<'a, 'm> {
    fn new(
        context: &'a Context<'m>, parser: &'a mut Parser<'m>,
        is_const: Result<&'a mut [bool], &'a [bool]>,
    ) -> Self {
        Self {
            context,
            parser,
            globals_len: context.globals.len(),
            is_const,
            is_body: false,
            locals: vec![],
            labels: vec![Label::default()],
        }
    }

    fn check_const(
        context: &'a Context<'m>, parser: &'a mut Parser<'m>, refs: &'a mut [bool],
        num_global_imports: usize, expected: ResultType<'m>,
    ) -> CheckResult {
        let mut expr = Expr::new(context, parser, Ok(refs));
        expr.globals_len = num_global_imports;
        expr.label().type_.results = expected;
        expr.check()
    }

    fn check_body(
        context: &'a Context<'m>, parser: &'a mut Parser<'m>, refs: &'a [bool],
        locals: Vec<ValType>, results: ResultType<'m>,
    ) -> CheckResult {
        let mut expr = Expr::new(context, parser, Err(refs));
        expr.is_body = true;
        expr.locals = locals;
        expr.label().type_.results = results;
        expr.check()
    }

    fn check(mut self) -> CheckResult {
        while !self.labels.is_empty() {
            self.instr()?;
        }
        Ok(())
    }

    fn instr(&mut self) -> CheckResult {
        use Instr::*;
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
            Loop(b) => self.push_label(self.blocktype(&b)?, LabelKind::Loop)?,
            If(b) => {
                self.pop_check(ValType::I32)?;
                self.push_label(self.blocktype(&b)?, LabelKind::If)?;
            }
            Else => {
                let label = self.label();
                check(core::mem::replace(&mut label.kind, LabelKind::Block) == LabelKind::If)?;
                let FuncType { params, results } = label.type_;
                self.pops(results)?;
                check(self.stack().is_empty())?;
                self.label().polymorphic = false;
                self.pushs(params);
            }
            End => unreachable!(),
            Br(l) => {
                self.pops(self.br_label(l)?)?;
                self.stack_polymorphic();
            }
            BrIf(l) => {
                self.pop_check(ValType::I32)?;
                self.swaps(self.br_label(l)?)?;
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
            ILoad(n, m) => self.load(NumType::i(n), n.into(), m)?,
            #[cfg(feature = "float-types")]
            FLoad(n, m) => self.load(NumType::f(n), n.into(), m)?,
            ILoad_(b, _, m) => self.load(NumType::i(b.into()), b.into(), m)?,
            IStore(n, m) => self.store(NumType::i(n), n.into(), m)?,
            #[cfg(feature = "float-types")]
            FStore(n, m) => self.store(NumType::f(n), n.into(), m)?,
            IStore_(b, m) => self.store(NumType::i(b.into()), b.into(), m)?,
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
            #[cfg(feature = "threads")]
            AtomicNotify(m) => {
                check(m.align == 2)?;
                self.pops([ValType::I32, ValType::I32][..].into())?;
                self.push(OpdType::I32);
            }
            #[cfg(feature = "threads")]
            AtomicWait(n, m) => {
                self.check_aligned(m, n.into())?;
                self.pops([ValType::I32, NumType::i(n).into(), ValType::I64][..].into())?;
                self.push(OpdType::I32);
            }
            #[cfg(feature = "threads")]
            AtomicFence => (),
            #[cfg(feature = "threads")]
            IAtomicLoad(n, m) => {
                self.check_aligned(m, n.into())?;
                self.load(NumType::i(n), n.into(), m)?
            }
            #[cfg(feature = "threads")]
            IAtomicLoad_(b, _, m) => {
                self.check_aligned(m, b.into())?;
                self.load(NumType::i(b.into()), b.into(), m)?;
            }
            #[cfg(feature = "threads")]
            IAtomicStore(n, m) => {
                self.check_aligned(m, n.into())?;
                self.store(NumType::i(n), n.into(), m)?;
            }
            #[cfg(feature = "threads")]
            IAtomicStore_(b, m) => {
                self.check_aligned(m, b.into())?;
                self.store(NumType::i(b.into()), b.into(), m)?;
            }
            #[cfg(feature = "threads")]
            AtomicOp(n, _, m) => {
                self.check_aligned(m, n.into())?;
                let num = NumType::i(n);
                self.pops([ValType::I32, num.into()][..].into())?;
                self.push(num.into());
            }
            #[cfg(feature = "threads")]
            AtomicOp_(b, _, _, m) => {
                self.check_aligned(m, b.into())?;
                let num = NumType::i(b.into());
                self.pops([ValType::I32, num.into()][..].into())?;
                self.push(NumType::i(b.into()).into())
            }
            #[cfg(feature = "threads")]
            AtomicExchange_(b, _, m) => {
                self.check_aligned(m, b.into())?;
                let num = NumType::i(b.into());
                self.pops([ValType::I32, num.into()][..].into())?;
                self.push(num.into())
            }
            #[cfg(feature = "threads")]
            AtomicExchange(n, m) => {
                self.check_aligned(m, n.into())?;
                self.pops([ValType::I32, NumType::i(n).into()][..].into())?;
                self.push(NumType::i(n).into())
            }
            #[cfg(feature = "threads")]
            AtomicCompareExchange(n, m) => {
                self.check_aligned(m, n.into())?;
                let num = NumType::i(n);
                self.pops([ValType::I32, num.into(), num.into()][..].into())?;
                self.push(num.into());
            }
            #[cfg(feature = "threads")]
            AtomicCompareExchange_(b, _, m) => {
                self.check_aligned(m, b.into())?;
                let num = NumType::i(b.into());
                self.pops([ValType::I32, num.into(), num.into()][..].into())?;
                self.push(num.into())
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

    fn label(&mut self) -> &mut Label<'m> {
        self.labels.last_mut().unwrap()
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

    fn push_label(&mut self, type_: FuncType<'m>, kind: LabelKind) -> CheckResult {
        self.pops(type_.params)?;
        let stack = type_.params.iter().cloned().map(OpdType::from).collect();
        let label = Label { type_, kind, polymorphic: false, stack };
        self.labels.push(label);
        Ok(())
    }

    fn end_label(&mut self) -> CheckResult {
        let label = self.label();
        if label.kind == LabelKind::If {
            check(label.type_.params == label.type_.results)?;
        }
        let results = label.type_.results;
        self.pops(results)?;
        check(self.labels.pop().unwrap().stack.is_empty())?;
        if !self.labels.is_empty() {
            self.pushs(results);
        }
        Ok(())
    }

    fn br_label(&self, l: LabelIdx) -> Result<ResultType<'m>, Error> {
        let l = l as usize;
        let n = self.labels.len();
        check(l < n)?;
        let label = &self.labels[n - l - 1];
        Ok(match label.kind {
            LabelKind::Block | LabelKind::If => label.type_.results,
            LabelKind::Loop => label.type_.params,
        })
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

    fn load(&mut self, t: NumType, n: usize, m: MemArg) -> CheckResult {
        check(!self.context.mems.is_empty())?;
        check(1 << m.align <= n / 8)?;
        self.pop_check(ValType::I32)?;
        self.push(t.into());
        Ok(())
    }

    fn store(&mut self, t: NumType, n: usize, m: MemArg) -> CheckResult {
        check(!self.context.mems.is_empty())?;
        check(1 << m.align <= n / 8)?;
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
    #[cfg(feature = "threads")]
    fn check_aligned(&mut self, m: MemArg, n: usize) -> CheckResult {
        check(1 << m.align == n / 8)?;
        Ok(())
    }
}
