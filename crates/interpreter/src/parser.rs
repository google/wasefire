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

#[cfg(feature = "debug")]
use crate::error::*;
use crate::syntax::*;
use crate::toctou::*;

#[derive(Debug, Default, Clone)]
pub struct Parser<'m, M: Mode> {
    data: &'m [u8],
    mode: PhantomData<M>,
}

impl<'m> Parser<'m, Check> {
    pub fn new(data: &'m [u8]) -> Self {
        Self::internal_new(data)
    }
}

impl<'m> Parser<'m, Use> {
    // Safety: The data must have been previously parsed in Check mode.
    pub unsafe fn new(data: &'m [u8]) -> Self {
        Self::internal_new(data)
    }
}

impl<'m, M: Mode> Parser<'m, M> {
    pub fn save(&self) -> &'m [u8] {
        self.data
    }

    // Safety: The data must have been saved in a similar state.
    pub unsafe fn restore(&mut self, data: &'m [u8]) {
        self.data = data;
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn check_len(&self, len: usize) -> MResult<(), M> {
        M::check(|| len <= self.data.len())
    }

    pub fn parse_bytes(&mut self, len: usize) -> MResult<&'m [u8], M> {
        self.check_len(len)?;
        let (result, data) = split_at(self.data, len);
        self.data = data;
        Ok(result)
    }

    pub fn split_at(&mut self, len: usize) -> MResult<Parser<'m, M>, M> {
        Ok(Self::internal_new(self.parse_bytes(len)?))
    }

    pub fn parse_byte(&mut self) -> MResult<u8, M> {
        Ok(get(self.parse_bytes(1)?, 0))
    }

    pub fn parse_leb128(&mut self, signed: bool, bits: u8) -> MResult<u64, M> {
        debug_assert!(bits <= 64);
        let mut val: u64 = 0;
        let mut len = bits;
        loop {
            let mut byte = self.parse_byte()? as u64;
            if byte & 0x80 == 0 {
                if signed && byte & 1 << core::cmp::min(len - 1, 6) != 0 {
                    if let Some(len) = len.checked_sub(7) {
                        val |= ((1 << len) - 1) << (bits - len);
                    } else {
                        M::check(|| (!byte & 0x7f) >> len == 0)?;
                        byte &= (1 << len) - 1;
                    }
                } else {
                    M::check(|| len >= 7 || byte >> len == 0)?;
                }
                val |= byte << (bits - len);
                break;
            }
            val |= (byte & 0x7f) << (bits - len);
            len = M::open(|| len.checked_sub(8))? + 1;
        }
        Ok(val)
    }

    pub fn parse_u32(&mut self) -> MResult<u32, M> {
        Ok(self.parse_leb128(false, 32)? as u32)
    }

    pub fn parse_s32(&mut self) -> MResult<i32, M> {
        Ok(self.parse_leb128(true, 32)? as i32)
    }

    pub fn parse_i32(&mut self) -> MResult<u32, M> {
        Ok(self.parse_s32()? as u32)
    }

    pub fn parse_s64(&mut self) -> MResult<i64, M> {
        Ok(self.parse_leb128(true, 64)? as i64)
    }

    pub fn parse_i64(&mut self) -> MResult<u64, M> {
        Ok(self.parse_s64()? as u64)
    }

    pub fn parse_typeidx(&mut self) -> MResult<TypeIdx, M> {
        self.parse_u32()
    }

    pub fn parse_funcidx(&mut self) -> MResult<TypeIdx, M> {
        self.parse_u32()
    }

    pub fn parse_tableidx(&mut self) -> MResult<TypeIdx, M> {
        self.parse_u32()
    }

    pub fn parse_memidx(&mut self) -> MResult<TypeIdx, M> {
        self.parse_u32()
    }

    pub fn parse_globalidx(&mut self) -> MResult<TypeIdx, M> {
        self.parse_u32()
    }

    pub fn parse_elemidx(&mut self) -> MResult<TypeIdx, M> {
        self.parse_u32()
    }

    pub fn parse_dataidx(&mut self) -> MResult<TypeIdx, M> {
        self.parse_u32()
    }

    pub fn parse_localidx(&mut self) -> MResult<TypeIdx, M> {
        self.parse_u32()
    }

    pub fn parse_labelidx(&mut self) -> MResult<TypeIdx, M> {
        self.parse_u32()
    }

    pub fn parse_section_id(&mut self) -> MResult<SectionId, M> {
        byte_enum::<M, _>(self.parse_byte()?)
    }

    pub fn split_section(&mut self) -> MResult<Parser<'m, M>, M> {
        let n = self.parse_u32()? as usize;
        self.split_at(n)
    }

    pub fn parse_name(&mut self) -> MResult<&'m str, M> {
        let n = self.parse_u32()? as usize;
        let xs = self.parse_bytes(n)?;
        M::choose(
            || core::str::from_utf8(xs).ok(),
            || unsafe { core::str::from_utf8_unchecked(xs) },
        )
    }

    pub fn parse_vec(&mut self) -> MResult<usize, M> {
        Ok(self.parse_u32()? as usize)
    }

    pub fn parse_valtype(&mut self) -> MResult<ValType, M> {
        byte_enum::<M, _>(self.parse_byte()?)
    }

    pub fn parse_resulttype(&mut self) -> MResult<ResultType<'m>, M> {
        let n = self.parse_vec()?;
        let xs = self.parse_bytes(n)?;
        M::check(|| xs.iter().all(|&x| ValType::try_from(x).is_ok()))?;
        // We don't have a safe version for this because we return a reference.
        let ptr = xs.as_ptr() as *const ValType;
        Ok(unsafe { core::slice::from_raw_parts(ptr, n) }.into())
    }

    pub fn parse_functype(&mut self) -> MResult<FuncType<'m>, M> {
        let byte = self.parse_byte()?;
        M::check(|| byte == 0x60)?;
        let params = self.parse_resulttype()?;
        let results = self.parse_resulttype()?;
        Ok(FuncType { params, results })
    }

    pub fn parse_reftype(&mut self) -> MResult<RefType, M> {
        byte_enum::<M, _>(self.parse_byte()?)
    }

    pub fn parse_limittype(&mut self) -> MResult<LimitType, M> {
        byte_enum::<M, LimitType>(self.parse_byte()?)
    }

    pub fn parse_limits(&mut self, mut max: u32) -> MResult<Limits, M> {
        let limit_type: LimitType = self.parse_limittype()?;

        let has_max = limit_type != LimitType::Unshared;
        let shared: bool = limit_type == LimitType::SharedWithMax;

        let min = self.parse_u32()?;
        if has_max {
            max = self.parse_u32()?;
        }

        Ok(Limits { shared, min, max })
    }

    pub fn parse_tabletype(&mut self) -> MResult<TableType, M> {
        let item = self.parse_reftype()?;
        let limits = self.parse_limits(TABLE_MAX)?;
        Ok(TableType { limits, item })
    }

    pub fn parse_memtype(&mut self) -> MResult<MemType, M> {
        self.parse_limits(MEM_MAX)
    }

    pub fn parse_mut(&mut self) -> MResult<Mut, M> {
        byte_enum::<M, _>(self.parse_byte()?)
    }

    pub fn parse_globaltype(&mut self) -> MResult<GlobalType, M> {
        let value = self.parse_valtype()?;
        let mutable = self.parse_mut()?;
        Ok(GlobalType { mutable, value })
    }

    pub fn parse_importdesc(&mut self) -> MResult<ImportDesc, M> {
        Ok(match self.parse_byte()? {
            0 => ImportDesc::Func(self.parse_typeidx()?),
            1 => ImportDesc::Table(self.parse_tabletype()?),
            2 => ImportDesc::Mem(self.parse_memtype()?),
            3 => ImportDesc::Global(self.parse_globaltype()?),
            _ => M::invalid()?,
        })
    }

    pub fn parse_exportdesc(&mut self) -> MResult<ExportDesc, M> {
        Ok(match self.parse_byte()? {
            0 => ExportDesc::Func(self.parse_funcidx()?),
            1 => ExportDesc::Table(self.parse_tableidx()?),
            2 => ExportDesc::Mem(self.parse_memidx()?),
            3 => ExportDesc::Global(self.parse_globalidx()?),
            _ => M::invalid()?,
        })
    }

    pub fn parse_memarg(&mut self) -> MResult<MemArg, M> {
        let align = self.parse_u32()?;
        let offset = self.parse_u32()?;
        Ok(MemArg { align, offset })
    }

    pub fn parse_blocktype(&mut self) -> MResult<BlockType, M> {
        let x = self.parse_leb128(true, 33)?;
        if x & 0x100000000 == 0 {
            return Ok(BlockType::Index(x as TypeIdx));
        }
        M::check(|| x & 0xffffff80 == 0xffffff80)?;
        let x = x as u8 & 0x7f;
        Ok(if x == 0x40 { BlockType::None } else { BlockType::Type(byte_enum::<M, _>(x)?) })
    }

    pub fn parse_br_table(&mut self) -> MResult<Instr<'m>, M> {
        let labels =
            (0 .. self.parse_u32()?).map(|_| self.parse_labelidx()).collect::<Result<_, _>>()?;
        Ok(Instr::BrTable(labels, self.parse_labelidx()?))
    }

    pub fn parse_instr(&mut self) -> MResult<Instr<'m>, M> {
        Ok(match self.parse_byte()? {
            0x00 => Instr::Unreachable,
            0x01 => Instr::Nop,
            0x02 => Instr::Block(self.parse_blocktype()?),
            0x03 => Instr::Loop(self.parse_blocktype()?),
            0x04 => Instr::If(self.parse_blocktype()?),
            0x05 => Instr::Else,
            0x0b => Instr::End,
            0x0c => Instr::Br(self.parse_labelidx()?),
            0x0d => Instr::BrIf(self.parse_labelidx()?),
            0x0e => self.parse_br_table()?,
            0x0f => Instr::Return,
            0x10 => Instr::Call(self.parse_funcidx()?),
            0x11 => {
                // For some reason, parsing order differs from field order here.
                let y = self.parse_typeidx()?;
                let x = self.parse_tableidx()?;
                Instr::CallIndirect(x, y)
            }
            0x1a => Instr::Drop,
            0x1b => Instr::Select(None),
            0x1c => Instr::Select(Some(self.parse_resulttype()?)),
            0x20 => Instr::LocalGet(self.parse_localidx()?),
            0x21 => Instr::LocalSet(self.parse_localidx()?),
            0x22 => Instr::LocalTee(self.parse_localidx()?),
            0x23 => Instr::GlobalGet(self.parse_globalidx()?),
            0x24 => Instr::GlobalSet(self.parse_globalidx()?),
            0x25 => Instr::TableGet(self.parse_tableidx()?),
            0x26 => Instr::TableSet(self.parse_tableidx()?),
            x @ 0x28 ..= 0x29 => Instr::ILoad((x - 0x28).into(), self.parse_memarg()?),
            x @ 0x2a ..= 0x2b => support_if!(
                "float-types"[x],
                Instr::FLoad((x - 0x2a).into(), self.parse_memarg()?),
                M::unsupported(if_debug!(Unsupported::Opcode(x)))?
            ),
            x @ 0x2c ..= 0x35 => Instr::ILoad_(
                ((x - 0x2c) / 2).into(),
                ((x - 0x2c) % 2).into(),
                self.parse_memarg()?,
            ),
            x @ 0x36 ..= 0x37 => Instr::IStore((x - 0x36).into(), self.parse_memarg()?),
            x @ 0x38 ..= 0x39 => support_if!(
                "float-types"[x],
                Instr::FStore((x - 0x38).into(), self.parse_memarg()?),
                M::unsupported(if_debug!(Unsupported::Opcode(x)))?
            ),
            x @ 0x3a ..= 0x3e => Instr::IStore_((x - 0x3a).into(), self.parse_memarg()?),
            0x3f => {
                check_eq::<M, _>(self.parse_byte()?, 0)?;
                Instr::MemorySize
            }
            0x40 => {
                check_eq::<M, _>(self.parse_byte()?, 0)?;
                Instr::MemoryGrow
            }
            0x41 => Instr::I32Const(self.parse_i32()?),
            0x42 => Instr::I64Const(self.parse_i64()?),
            0x43 => support_if!(
                "float-types"[],
                Instr::F32Const(u32::from_le_bytes(self.parse_bytes(4)?.try_into().unwrap())),
                M::unsupported(if_debug!(Unsupported::Opcode(0x43)))?
            ),
            0x44 => support_if!(
                "float-types"[],
                Instr::F64Const(u64::from_le_bytes(self.parse_bytes(8)?.try_into().unwrap())),
                M::unsupported(if_debug!(Unsupported::Opcode(0x44)))?
            ),
            x @ 0x45 ..= 0x5a => {
                let n = Nx::from((x - 0x45) / 11);
                match (x - 0x45) % 11 {
                    0 => Instr::ITestOp(n, ITestOp::Eqz),
                    y => Instr::IRelOp(n, (y - 1).into()),
                }
            }
            x @ 0x5b ..= 0x66 => support_if!(
                "float-types"[x],
                Instr::FRelOp(((x - 0x5b) / 6).into(), ((x - 0x5b) % 6).into()),
                M::unsupported(if_debug!(Unsupported::Opcode(x)))?
            ),
            x @ 0x67 ..= 0x8a => {
                let n = Nx::from((x - 0x67) / 18);
                match (x - 0x67) % 18 {
                    y @ (0 ..= 2) => Instr::IUnOp(n, y.into()),
                    y => Instr::IBinOp(n, (y - 3).into()),
                }
            }
            x @ 0x8b ..= 0xa6 => support_if!(
                "float-types"[x],
                {
                    let n = Nx::from((x - 0x8b) / 14);
                    match (x - 0x8b) % 14 {
                        y @ (0 ..= 6) => Instr::FUnOp(n, y.into()),
                        y => Instr::FBinOp(n, (y - 7).into()),
                    }
                },
                M::unsupported(if_debug!(Unsupported::Opcode(x)))?
            ),
            0xa7 => Instr::CvtOp(CvtOp::Wrap),
            x @ 0xa8 ..= 0xab => support_if!(
                "float-types"[x],
                Instr::CvtOp(CvtOp::Trunc(
                    Nx::N32,
                    ((x - 0xa8) / 2).into(),
                    ((x - 0xa8) % 2).into(),
                )),
                M::unsupported(if_debug!(Unsupported::Opcode(x)))?
            ),
            x @ 0xac ..= 0xad => Instr::CvtOp(CvtOp::Extend((x - 0xac).into())),
            x @ 0xae ..= 0xb1 => support_if!(
                "float-types"[x],
                Instr::CvtOp(CvtOp::Trunc(
                    Nx::N64,
                    ((x - 0xae) / 2).into(),
                    ((x - 0xae) % 2).into(),
                )),
                M::unsupported(if_debug!(Unsupported::Opcode(x)))?
            ),
            x @ 0xb2 ..= 0xbb => support_if!(
                "float-types"[x],
                {
                    let n = Nx::from((x - 0xb2) / 5);
                    match (x - 0xb2) % 5 {
                        4 => match n {
                            Nx::N32 => Instr::CvtOp(CvtOp::Demote),
                            Nx::N64 => Instr::CvtOp(CvtOp::Promote),
                        },
                        y => Instr::CvtOp(CvtOp::Convert(n, (y / 2).into(), (y % 2).into())),
                    }
                },
                M::unsupported(if_debug!(Unsupported::Opcode(x)))?
            ),
            x @ 0xbc ..= 0xbd => support_if!(
                "float-types"[x],
                Instr::CvtOp(CvtOp::IReinterpret((x - 0xbc).into())),
                M::unsupported(if_debug!(Unsupported::Opcode(x)))?
            ),
            x @ 0xbe ..= 0xbf => support_if!(
                "float-types"[x],
                Instr::CvtOp(CvtOp::FReinterpret((x - 0xbe).into())),
                M::unsupported(if_debug!(Unsupported::Opcode(x)))?
            ),
            x @ 0xc0 ..= 0xc4 => Instr::IExtend((x - 0xc0).into()),
            0xd0 => Instr::RefNull(self.parse_reftype()?),
            0xd1 => Instr::RefIsNull,
            0xd2 => Instr::RefFunc(self.parse_funcidx()?),
            0xfc => match self.parse_u32()? {
                x @ 0 ..= 7 => support_if!(
                    "float-types"[x],
                    Instr::CvtOp(CvtOp::TruncSat(
                        ((x & 4 != 0) as u8).into(),
                        ((x & 2 != 0) as u8).into(),
                        ((x & 1) as u8).into(),
                    )),
                    M::unsupported(if_debug!(Unsupported::OpcodeFc(x)))?
                ),
                8 => {
                    let x = self.parse_dataidx()?;
                    check_eq::<M, _>(self.parse_byte()?, 0)?;
                    Instr::MemoryInit(x)
                }
                9 => Instr::DataDrop(self.parse_dataidx()?),
                10 => {
                    check_eq::<M, _>(self.parse_bytes(2)?, &[0; 2])?;
                    Instr::MemoryCopy
                }
                11 => {
                    check_eq::<M, _>(self.parse_byte()?, 0)?;
                    Instr::MemoryFill
                }
                12 => {
                    // For some reason, parsing order differs from field order here.
                    let y = self.parse_elemidx()?;
                    let x = self.parse_tableidx()?;
                    Instr::TableInit(x, y)
                }
                13 => Instr::ElemDrop(self.parse_elemidx()?),
                14 => Instr::TableCopy(self.parse_tableidx()?, self.parse_tableidx()?),
                15 => Instr::TableGrow(self.parse_tableidx()?),
                16 => Instr::TableSize(self.parse_tableidx()?),
                17 => Instr::TableFill(self.parse_tableidx()?),
                _ => M::invalid()?,
            },
            0xfd => support_if!(
                "vector-types"[],
                unimplemented!(),
                M::unsupported(if_debug!(Unsupported::Opcode(0xfd)))?
            ),
            0xfe => support_if!(
                "threads"[],
                match self.parse_byte()? {
                    0x00 => Instr::AtomicNotify(self.parse_memarg()?), // memory.atomic.notify
                    0x01 => Instr::AtomicWait(Nx::N32, self.parse_memarg()?), // memory.atomic.wait32
                    0x02 => Instr::AtomicWait(Nx::N64, self.parse_memarg()?), // memory.atomic.wait64
                    0x03 => {check_eq::<M, _>(self.parse_byte()?, 0)?; Instr::AtomicFence()}, // atomic.fence
                    0x10 => Instr::IAtomicLoad(Nx::N32, self.parse_memarg()?), // i32.atomic.load
                    0x11 => Instr::IAtomicLoad(Nx::N64, self.parse_memarg()?), // i64.atomic.load
                    x@0x12 ..= 0x16 => {
                        let b = Bx::from(x-0x12 );
                        Instr::IAtomicLoad_(b, Sx::U, self.parse_memarg()?)}, // iXX.atomic.loadXX_u
                    0x17 => Instr::IAtomicStore(Nx::N32, self.parse_memarg()?), // i32.atomic.store
                    0x18 => Instr::IAtomicStore(Nx::N64, self.parse_memarg()?), // i64.atomic.store
                    x@0x19 ..= 0x1D => {
                        let b = Bx::from(x- 0x19 );
                        Instr::IAtomicStore_(b, self.parse_memarg()?)}, // iXX.atomic.storeXX
                    0x1E => Instr::IAtomicBinOp(IBinOp::Add, Nx::N32, self.parse_memarg()?),  // i32.atomic.add
                    0x1F => Instr::IAtomicBinOp(IBinOp::Add, Nx::N64, self.parse_memarg()?),  // i64.atomic.add
                    x@0x20 ..= 0x24 => {
                        let b = Bx::from(x-0x20 );
                        Instr::IAtomicBinOp_(IBinOp::Sub, b, Sx::U, self.parse_memarg()?)}, // iXX.atomic.addXX_u
                    0x25 => Instr::IAtomicBinOp(IBinOp::Sub, Nx::N32, self.parse_memarg()?),  // i32.atomic.sub
                    0x26 => Instr::IAtomicBinOp(IBinOp::Sub, Nx::N64, self.parse_memarg()?),  // i64.atomic.sub
                    x@0x27 ..= 0x2B => {
                        let b = Bx::from(x-0x27 );
                        Instr::IAtomicBinOp_(IBinOp::Sub, b, Sx::U, self.parse_memarg()?)}, // iXX.atomic.subXX_u
                    0x2c => Instr::IAtomicBinOp(IBinOp::And, Nx::N32, self.parse_memarg()?),  // i32.atomic.and
                    0x2d => Instr::IAtomicBinOp(IBinOp::And, Nx::N64, self.parse_memarg()?),  // i64.atomic.and
                    x@0x2e ..= 0x32 => {
                        let b = Bx::from(x-0x2e );
                        Instr::IAtomicBinOp_(IBinOp::And, b, Sx::U, self.parse_memarg()?)}, // iXX.atomic.andXX_u
                    0x33 => Instr::IAtomicBinOp(IBinOp::Or, Nx::N32, self.parse_memarg()?),  // i32.atomic.or
                    0x34 => Instr::IAtomicBinOp(IBinOp::Or, Nx::N64, self.parse_memarg()?),  // i64.atomic.or
                    x@0x35 ..= 0x39 => {
                        let b = Bx::from(x-0x35 );
                        Instr::IAtomicBinOp_(IBinOp::Or, b, Sx::U, self.parse_memarg()?)}, // iXX.atomic.orXX_u
                    0x3A => Instr::IAtomicBinOp(IBinOp::Xor, Nx::N32, self.parse_memarg()?),  // i32.atomic.xor
                    0x3B => Instr::IAtomicBinOp(IBinOp::Xor, Nx::N64, self.parse_memarg()?),  // i64.atomic.xor
                    x@0x3C ..= 0x40 => {
                        let b = Bx::from(x - 0x3C );
                        Instr::IAtomicBinOp_(IBinOp::Xor, b, Sx::U, self.parse_memarg()?)
                    }, // iXX.atomic.xorXX_u
                    0x41 => Instr::AtomicExchange(Nx::N32, self.parse_memarg()?),  // i32.atomic.rmw.xchg
                    0x42 => Instr::AtomicExchange(Nx::N64, self.parse_memarg()?),  // i64.atomic.rmw.xchg
                    x@0x43 ..= 0x47 => {
                        let b = Bx::from(x - 0x43 );
                        Instr::AtomicExchange_( b, Sx::U, self.parse_memarg()?)
                    },  // iXX.atomic.rmwXX.xchg_u
                    0x48 => Instr::AtomicCompareExchange(Nx::N32, self.parse_memarg()?), // i32.atomic.cmpxchg
                    0x49 => Instr::AtomicCompareExchange(Nx::N64, self.parse_memarg()?), // i64.atomic.cmpxchg
                    x@0x4A ..= 0x4E => {
                        let b = Bx::from(x - 0x4A);
                        Instr::AtomicCompareExchange_(b, Sx::U, self.parse_memarg()?)
                    }, // iXX.atomic.xorXX_u
                    _x => {
                        M::unsupported(if_debug!(Unsupported::Opcode(_x)))?
                    }
                },
                M::unsupported(if_debug!(Unsupported::Opcode(0xfe)))?
            ),
            _i => {
                #[cfg(feature = "debug")]
                println!("Invalid instruction {:#06x}", _i);
                M::invalid()?
            }
        })
    }

    pub fn parse_locals(&mut self, locals: &mut Vec<ValType>) -> MResult<(), M> {
        for _ in 0 .. self.parse_vec()? {
            let len = self.parse_u32()? as usize;
            if locals.len().checked_add(len).map_or(true, |x| x > MAX_LOCALS) {
                return M::unsupported(if_debug!(Unsupported::MaxLocals));
            }
            let val = self.parse_valtype()?;
            locals.extend(core::iter::repeat(val).take(len));
        }
        Ok(())
    }

    pub fn parse_elem(&mut self, user: &mut impl ParseElem<'m, M>) -> MResult<(), M> {
        // A <---- B ----> <- C --> <--- D ---->
        // 0          expr          vec(funcidx)
        // 1               elemkind vec(funcidx)
        // 2 tableidx expr elemkind vec(funcidx)
        // 3               elemkind vec(funcidx)
        // 4          expr          vec(expr)
        // 5               reftype  vec(expr)
        // 6 tableidx expr reftype  vec(expr)
        // 7               reftype  vec(expr)

        // A: We parse the flag.
        let flags = self.parse_u32()?;
        M::check(|| flags < 8)?;
        // B: We parse the mode.
        let mode = if flags & 1 == 0 {
            let table = if flags & 2 == 0 { 0 } else { self.parse_tableidx()? };
            ElemMode::Active { table, offset: self }
        } else if flags & 2 == 0 {
            ElemMode::Passive
        } else {
            ElemMode::Declarative
        };
        user.mode(mode)?;
        // C: We parse the type.
        let type_ = if flags & 3 == 0 {
            RefType::FuncRef
        } else if flags & 4 == 0 {
            let kind = self.parse_byte()?;
            M::check(|| kind == 0)?;
            RefType::FuncRef
        } else {
            self.parse_reftype()?
        };
        user.type_(type_)?;
        // D: We parse the init.
        if flags & 4 == 0 {
            for _ in 0 .. self.parse_u32()? {
                user.init_funcidx(self.parse_funcidx()?)?;
            }
        } else {
            for _ in 0 .. self.parse_u32()? {
                user.init_expr(self)?;
            }
        }
        Ok(())
    }

    pub fn parse_data(&mut self, user: &mut impl ParseData<'m, M>) -> MResult<(), M> {
        // A < B -> <C > <-- D -->
        // 0        expr vec(byte)
        // 1             vec(byte)
        // 2 memidx expr vec(byte)

        // A: We parse the flag.
        let flags = self.parse_u32()?;
        M::check(|| flags < 3)?;
        // B: We parse the mode.
        let memory = if flags & 2 == 0 { 0 } else { self.parse_memidx()? };
        let mode = if flags & 1 == 0 {
            DataMode::Active { memory, offset: self }
        } else {
            DataMode::Passive
        };
        user.mode(mode)?;
        // D: We parse the init.
        let len = self.parse_u32()? as usize;
        user.init(self.parse_bytes(len)?)
    }

    pub fn skip_to_else(&mut self) -> MResult<(), M> {
        let mut depth = 0;
        loop {
            let saved = self.save();
            match self.parse_instr()? {
                Instr::Block(_) => depth += 1,
                Instr::Loop(_) => depth += 1,
                Instr::If(_) => depth += 1,
                Instr::End if depth == 0 => {
                    unsafe { self.restore(saved) };
                    return Ok(());
                }
                Instr::End => depth -= 1,
                Instr::Else if depth == 0 => return Ok(()),
                _ => (),
            }
        }
    }

    pub fn skip_to_end(&mut self, l: LabelIdx) -> MResult<(), M> {
        let mut depth = l as usize + 1;
        while depth > 0 {
            match self.parse_instr()? {
                Instr::Block(_) => depth += 1,
                Instr::Loop(_) => depth += 1,
                Instr::If(_) => depth += 1,
                Instr::End => depth -= 1,
                _ => (),
            }
        }
        Ok(())
    }
}

pub trait ParseElem<'m, M: Mode> {
    // Must read an expr from the parser.
    fn mode(&mut self, mode: ElemMode<'_, 'm, M>) -> MResult<(), M> {
        match mode {
            ElemMode::Active { offset, .. } => offset.skip_to_end(0),
            _ => Ok(()),
        }
    }

    fn type_(&mut self, _type: RefType) -> MResult<(), M> {
        Ok(())
    }

    fn init_funcidx(&mut self, _x: FuncIdx) -> MResult<(), M> {
        Ok(())
    }

    // Must read an expr from the parser.
    fn init_expr(&mut self, parser: &mut Parser<'m, M>) -> MResult<(), M> {
        parser.skip_to_end(0)
    }
}

#[derive(Debug)]
pub enum ElemMode<'a, 'm, M: Mode> {
    Passive,
    Active { table: TableIdx, offset: &'a mut Parser<'m, M> },
    Declarative,
}

pub struct SkipElem;

impl<'m, M: Mode> ParseElem<'m, M> for SkipElem {}

pub trait ParseData<'m, M: Mode> {
    // Must read an expr from the parser.
    fn mode(&mut self, mode: DataMode<'_, 'm, M>) -> MResult<(), M> {
        match mode {
            DataMode::Passive => Ok(()),
            DataMode::Active { offset, .. } => offset.skip_to_end(0),
        }
    }

    fn init(&mut self, _init: &'m [u8]) -> MResult<(), M> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum DataMode<'a, 'm, M: Mode> {
    Passive,
    Active { memory: MemIdx, offset: &'a mut Parser<'m, M> },
}

pub struct SkipData;

impl<'m, M: Mode> ParseData<'m, M> for SkipData {}

impl<'m, M: Mode> Parser<'m, M> {
    fn internal_new(data: &'m [u8]) -> Self {
        Self { data, mode: PhantomData }
    }
}

/// Maximum number of locals (must be less than 2^32).
// NOTE: This should be configurable.
const MAX_LOCALS: usize = 100;

fn check_eq<M: Mode, T: Eq>(x: T, y: T) -> MResult<(), M> {
    M::check(|| x == y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    pub fn parse_leb128_is_correct() {
        #[track_caller]
        fn test(signed: bool, bits: u8, data: &[u8], result: Result<u64, Error>) {
            let mut parser = <Parser<Check>>::new(data);
            assert_eq!(parser.parse_leb128(signed, bits), result);
            assert!(parser.is_empty());
            if let Ok(result) = result {
                let mut parser = unsafe { <Parser<Use>>::new(data) };
                assert_eq!(parser.parse_leb128(signed, bits).into_ok(), result);
                assert!(parser.is_empty());
            }
        }

        test(false, 8, &[], Err(Error::Invalid));
        test(false, 8, &[0x00], Ok(0));
        test(false, 8, &[0x80, 0x00], Ok(0));
        test(false, 8, &[0x80, 0x80], Err(Error::Invalid));
        test(false, 8, &[0x03], Ok(3));
        test(false, 8, &[0x83, 0x00], Ok(3));
        test(false, 8, &[0x83, 0x01], Ok(0x83));
        test(false, 8, &[0x83, 0x02], Err(Error::Invalid));
        test(false, 8, &[0x83, 0x7f], Err(Error::Invalid));
        test(true, 16, &[], Err(Error::Invalid));
        test(true, 16, &[0x7f], Ok(0xffff));
        test(true, 16, &[0xff, 0xff, 0x7f], Ok(0xffff));
        test(true, 16, &[0xff, 0xff, 0xff], Err(Error::Invalid));
        test(true, 16, &[0x7e], Ok(0xfffe));
        test(true, 16, &[0xfe, 0x7f], Ok(0xfffe));
        test(true, 16, &[0xfe, 0xff, 0x7f], Ok(0xfffe));
        test(true, 16, &[0x03], Ok(3));
        test(true, 16, &[0x83, 0x00], Ok(3));
        test(true, 16, &[0x83, 0x80, 0x01], Ok(0x4003));
        test(true, 16, &[0x83, 0x80, 0x02], Err(Error::Invalid));
        test(true, 16, &[0x83, 0x80, 0x7c], Err(Error::Invalid));
        test(true, 16, &[0x83, 0x80, 0x7e], Ok(0x8003));
        test(false, 32, &[], Err(Error::Invalid));
        test(false, 32, &[0x00], Ok(0));
        test(false, 32, &[0x80, 0x00], Ok(0));
        test(false, 32, &[0x80, 0x80, 0x80, 0x80, 0x08], Ok(0x80000000));
        test(true, 32, &[0x80, 0x80, 0x80, 0x80, 0x08], Err(Error::Invalid));
        test(true, 32, &[0x80, 0x80, 0x80, 0x80, 0x78], Ok(0x80000000));
        test(false, 32, &[0x80], Err(Error::Invalid));
        test(false, 32, &[0x80, 0x01], Ok(0x80));
        test(false, 32, &[0x80, 0x80, 0x80, 0x80, 0x10], Err(Error::Invalid));
        test(false, 32, &[0xff, 0xff, 0xff, 0xff, 0x0f], Ok(0xffffffff));
        test(false, 32, &[0xff, 0xff, 0xff, 0xff, 0x7f], Err(Error::Invalid));
        test(true, 32, &[0xff, 0xff, 0xff, 0xff, 0x7f], Ok(0xffffffff));
    }
}
