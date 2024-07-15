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
use core::ops::Deref;

use num_enum::{TryFromPrimitive, UnsafeFromPrimitive};
#[cfg(feature = "threads")]
use portable_atomic::{AtomicU16, AtomicU32, AtomicU64, AtomicU8, Ordering};

#[derive(Debug, Copy, Clone, PartialEq, Eq, TryFromPrimitive, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum NumType {
    I32 = 0x7f,
    I64 = 0x7e,
    F32 = 0x7d,
    F64 = 0x7c,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, TryFromPrimitive, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum VecType {
    V128 = 0x7b,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, TryFromPrimitive, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum RefType {
    FuncRef = 0x70,
    ExternRef = 0x6f,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, TryFromPrimitive, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum ValType {
    I32 = 0x7f,
    I64 = 0x7e,
    F32 = 0x7d,
    F64 = 0x7c,
    V128 = 0x7b,
    FuncRef = 0x70,
    ExternRef = 0x6f,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ResultType<'m>(&'m [ValType]);

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct FuncType<'m> {
    pub params: ResultType<'m>,
    pub results: ResultType<'m>,
}

#[cfg(feature = "threads")]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Share {
    Unshared,
    Shared,
}

#[cfg(feature = "threads")]
impl From<u8> for Share {
    fn from(x: u8) -> Self {
        match x {
            0 => Share::Unshared,
            1 => Share::Shared,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Limits {
    pub min: u32,
    pub max: u32,
    #[cfg(feature = "threads")]
    pub share: Share,
}

pub const TABLE_MAX: u32 = u32::MAX;
pub const MEM_MAX: u32 = 0x10000;

impl Limits {
    pub fn valid(&self, k: u32) -> bool {
        self.min <= self.max && self.max <= k
    }

    pub fn matches(self, other: Limits) -> bool {
        self.min >= other.min && self.max <= other.max
    }
}

pub type MemType = Limits;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TableType {
    pub limits: Limits,
    pub item: RefType,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, TryFromPrimitive, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum Mut {
    Const = 0,
    Var = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GlobalType {
    pub mutable: Mut,
    pub value: ValType,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sx {
    U,
    S,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Nx {
    N32,
    N64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Bx {
    N32B8,
    N32B16,
    N64B8,
    N64B16,
    N64B32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ITestOp {
    Eqz,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IRelOp {
    Eq,
    Ne,
    Lt(Sx),
    Gt(Sx),
    Le(Sx),
    Ge(Sx),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IUnOp {
    Clz,
    Ctz,
    PopCnt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IBinOp {
    Add,
    Sub,
    Mul,
    Div(Sx),
    Rem(Sx),
    And,
    Or,
    Xor,
    Shl,
    Shr(Sx),
    Rotl,
    Rotr,
}

#[cfg(feature = "float-types")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FRelOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

#[cfg(feature = "float-types")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FUnOp {
    Abs,
    Neg,
    Ceil,
    Floor,
    Trunc,
    Nearest,
    Sqrt,
}

#[cfg(feature = "float-types")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Min,
    Max,
    CopySign,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CvtOp {
    Wrap,
    Extend(Sx),
    #[cfg(feature = "float-types")]
    Trunc(Nx, Nx, Sx),
    #[cfg(feature = "float-types")]
    TruncSat(Nx, Nx, Sx),
    #[cfg(feature = "float-types")]
    Convert(Nx, Nx, Sx),
    #[cfg(feature = "float-types")]
    Demote,
    #[cfg(feature = "float-types")]
    Promote,
    #[cfg(feature = "float-types")]
    IReinterpret(Nx),
    #[cfg(feature = "float-types")]
    FReinterpret(Nx),
}

#[cfg(feature = "threads")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtomicUnOp {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Xchg,
}

#[cfg(feature = "threads")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtomicBinOp {
    Cmpxchg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instr<'m> {
    Unreachable,
    Nop,
    Block(BlockType),
    Loop(BlockType),
    If(BlockType),
    Else,
    End,
    Br(LabelIdx),
    BrIf(LabelIdx),
    BrTable(Vec<LabelIdx>, LabelIdx),
    Return,
    Call(FuncIdx),
    CallIndirect(TableIdx, TypeIdx), // parsing order differs
    Drop,
    Select(Option<ResultType<'m>>),
    LocalGet(LocalIdx),
    LocalSet(LocalIdx),
    LocalTee(LocalIdx),
    GlobalGet(GlobalIdx),
    GlobalSet(GlobalIdx),
    TableGet(TableIdx),
    TableSet(TableIdx),
    ILoad(Nx, MemArg),
    #[cfg(feature = "float-types")]
    FLoad(Nx, MemArg),
    ILoad_(Bx, Sx, MemArg),
    IStore(Nx, MemArg),
    #[cfg(feature = "float-types")]
    FStore(Nx, MemArg),
    IStore_(Bx, MemArg),
    MemorySize,
    MemoryGrow,
    I32Const(u32),
    I64Const(u64),
    #[cfg(feature = "float-types")]
    F32Const(u32),
    #[cfg(feature = "float-types")]
    F64Const(u64),
    ITestOp(Nx, ITestOp),
    IRelOp(Nx, IRelOp),
    #[cfg(feature = "float-types")]
    FRelOp(Nx, FRelOp),
    IUnOp(Nx, IUnOp),
    #[cfg(feature = "float-types")]
    FUnOp(Nx, FUnOp),
    IBinOp(Nx, IBinOp),
    #[cfg(feature = "float-types")]
    FBinOp(Nx, FBinOp),
    CvtOp(CvtOp),
    IExtend(Bx),
    RefNull(RefType),
    RefIsNull,
    RefFunc(FuncIdx),
    MemoryInit(DataIdx),
    DataDrop(DataIdx),
    MemoryCopy,
    MemoryFill,
    TableInit(TableIdx, ElemIdx), // parsing order differs
    ElemDrop(ElemIdx),
    TableCopy(TableIdx, TableIdx),
    TableGrow(TableIdx),
    TableSize(TableIdx),
    TableFill(TableIdx),
    #[cfg(feature = "threads")]
    AtomicNotify(MemArg),
    #[cfg(feature = "threads")]
    AtomicWait(Nx, MemArg),
    #[cfg(feature = "threads")]
    AtomicFence,
    #[cfg(feature = "threads")]
    AtomicLoad(Nx, MemArg),
    #[cfg(feature = "threads")]
    AtomicLoad_(Bx, MemArg),
    #[cfg(feature = "threads")]
    AtomicStore(Nx, MemArg),
    #[cfg(feature = "threads")]
    AtomicStore_(Bx, MemArg),
    #[cfg(feature = "threads")]
    AtomicUnOp(Nx, AtomicUnOp, MemArg),
    #[cfg(feature = "threads")]
    AtomicUnOp_(Bx, AtomicUnOp, MemArg),
    #[cfg(feature = "threads")]
    AtomicBinOp(Nx, AtomicBinOp, MemArg),
    #[cfg(feature = "threads")]
    AtomicBinOp_(Bx, AtomicBinOp, MemArg),
}

pub type TypeIdx = u32;
pub type FuncIdx = u32;
pub type TableIdx = u32;
pub type MemIdx = u32;
pub type GlobalIdx = u32;
pub type ElemIdx = u32;
pub type DataIdx = u32;
pub type LocalIdx = u32;
pub type LabelIdx = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportDesc {
    Func(FuncIdx),
    Table(TableIdx),
    Mem(MemIdx),
    Global(GlobalIdx),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

#[derive(Debug, Clone)]
pub struct Import<'m> {
    pub module: &'m str,
    pub name: &'m str,
    pub desc: ImportDesc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternType<'m> {
    Func(FuncType<'m>),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

impl<'m> ExternType<'m> {
    pub fn matches(&self, other: &ExternType<'m>) -> bool {
        match (self, other) {
            (ExternType::Func(x), ExternType::Func(y)) => x == y,
            (
                ExternType::Table(TableType { limits: t, item: x }),
                ExternType::Table(TableType { limits: s, item: y }),
            ) => t.matches(*s) && x == y,
            (ExternType::Mem(t), ExternType::Mem(s)) => t.matches(*s),
            (ExternType::Global(x), ExternType::Global(y)) => x == y,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockType {
    None,
    Type(ValType),
    Index(TypeIdx),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MemArg {
    pub align: u32,
    pub offset: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, TryFromPrimitive, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum SectionId {
    Custom = 0,
    Type = 1,
    Import = 2,
    Function = 3,
    Table = 4,
    Memory = 5,
    Global = 6,
    Export = 7,
    Start = 8,
    Element = 9,
    Code = 10,
    Data = 11,
    DataCount = 12,
}

impl<'m> Deref for ResultType<'m> {
    type Target = [ValType];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'m> From<&'m [ValType]> for ResultType<'m> {
    fn from(xs: &'m [ValType]) -> Self {
        Self(xs)
    }
}

impl<'m> From<ValType> for ResultType<'m> {
    fn from(x: ValType) -> Self {
        macro_rules! make {
            ($($x:ident),*) => {{
                static VAL_TYPES: &[ValType] = &[$(ValType::$x),*];
                make!([][0][$($x)*])
            }};
            ([$($output:tt)*][$i:expr][]) => {
                Self(match x { $($output)* })
            };
            ([$($output:tt)*][$i:expr][$x:ident $($input:ident)*]) => {
                make!([$($output)* ValType::$x => core::slice::from_ref(&VAL_TYPES[$i]),]
                      [$i + 1][$($input)*])
            };
        }
        make!(I32, I64, F32, F64, V128, FuncRef, ExternRef)
    }
}

impl<'m> From<()> for ResultType<'m> {
    fn from(_: ()) -> Self {
        Self(&[])
    }
}

impl NumType {
    pub fn i(n: Nx) -> Self {
        match n {
            Nx::N32 => NumType::I32,
            Nx::N64 => NumType::I64,
        }
    }

    #[cfg(feature = "float-types")]
    pub fn f(n: Nx) -> Self {
        match n {
            Nx::N32 => NumType::F32,
            Nx::N64 => NumType::F64,
        }
    }
}

impl From<Nx> for usize {
    fn from(x: Nx) -> Self {
        match x {
            Nx::N32 => 32,
            Nx::N64 => 64,
        }
    }
}

impl From<Bx> for Nx {
    fn from(x: Bx) -> Self {
        match x {
            Bx::N32B8 | Bx::N32B16 => Nx::N32,
            Bx::N64B8 | Bx::N64B16 | Bx::N64B32 => Nx::N64,
        }
    }
}

impl From<Bx> for usize {
    fn from(x: Bx) -> Self {
        match x {
            Bx::N32B8 | Bx::N64B8 => 8,
            Bx::N32B16 | Bx::N64B16 => 16,
            Bx::N64B32 => 32,
        }
    }
}

impl CvtOp {
    pub fn dst(&self) -> NumType {
        match *self {
            CvtOp::Wrap => NumType::I32,
            CvtOp::Extend(_) => NumType::I64,
            #[cfg(feature = "float-types")]
            CvtOp::Trunc(n, _, _) => NumType::i(n),
            #[cfg(feature = "float-types")]
            CvtOp::TruncSat(n, _, _) => NumType::i(n),
            #[cfg(feature = "float-types")]
            CvtOp::Convert(n, _, _) => NumType::f(n),
            #[cfg(feature = "float-types")]
            CvtOp::Demote => NumType::F32,
            #[cfg(feature = "float-types")]
            CvtOp::Promote => NumType::F64,
            #[cfg(feature = "float-types")]
            CvtOp::IReinterpret(n) => NumType::i(n),
            #[cfg(feature = "float-types")]
            CvtOp::FReinterpret(n) => NumType::f(n),
        }
    }

    pub fn src(&self) -> NumType {
        match *self {
            CvtOp::Wrap => NumType::I64,
            CvtOp::Extend(_) => NumType::I32,
            #[cfg(feature = "float-types")]
            CvtOp::Trunc(_, m, _) => NumType::f(m),
            #[cfg(feature = "float-types")]
            CvtOp::TruncSat(_, m, _) => NumType::f(m),
            #[cfg(feature = "float-types")]
            CvtOp::Convert(_, m, _) => NumType::i(m),
            #[cfg(feature = "float-types")]
            CvtOp::Demote => NumType::F64,
            #[cfg(feature = "float-types")]
            CvtOp::Promote => NumType::F32,
            #[cfg(feature = "float-types")]
            CvtOp::IReinterpret(n) => NumType::f(n),
            #[cfg(feature = "float-types")]
            CvtOp::FReinterpret(n) => NumType::i(n),
        }
    }
}

#[cfg(feature = "threads")]
macro_rules! impl_atomic_op {
    ($u:ident) => {
        impl AtomicUnOp {
            pub fn $u(&self, ptr: *mut $u, val: $u) -> $u {
                let x = unsafe { paste::paste!([<Atomic $u:upper>]::from_ptr(ptr)) };
                match self {
                    AtomicUnOp::Add => x.fetch_add(val, Ordering::SeqCst),
                    AtomicUnOp::Sub => x.fetch_sub(val, Ordering::SeqCst),
                    AtomicUnOp::And => x.fetch_and(val, Ordering::SeqCst),
                    AtomicUnOp::Or => x.fetch_or(val, Ordering::SeqCst),
                    AtomicUnOp::Xor => x.fetch_xor(val, Ordering::SeqCst),
                    AtomicUnOp::Xchg => x.swap(val, Ordering::SeqCst),
                }
            }
        }
        impl AtomicBinOp {
            pub fn $u(&self, ptr: *mut $u, val1: $u, val2: $u) -> $u {
                let x = unsafe { paste::paste!([<Atomic $u:upper>]::from_ptr(ptr)) };
                match x.compare_exchange(val1, val2, Ordering::SeqCst, Ordering::SeqCst) {
                    Err(v) => v,
                    Ok(v) => v
                }
            }
        }
    };
}

#[cfg(feature = "threads")]
impl_atomic_op!(u64);
#[cfg(feature = "threads")]
impl_atomic_op!(u32);
#[cfg(feature = "threads")]
impl_atomic_op!(u16);
#[cfg(feature = "threads")]
impl_atomic_op!(u8);

macro_rules! impl_op {
    ($n:ident, $u:ident, $i:ident, $f:ident) => {
        impl ITestOp {
            pub fn $n(&self, x: $u) -> bool {
                match self {
                    ITestOp::Eqz => x == 0,
                }
            }
        }

        impl IRelOp {
            pub fn $n(&self, x: $u, y: $u) -> bool {
                match self {
                    IRelOp::Eq => x == y,
                    IRelOp::Ne => x != y,
                    IRelOp::Lt(Sx::U) => x < y,
                    IRelOp::Lt(Sx::S) => (x as $i) < (y as $i),
                    IRelOp::Gt(Sx::U) => x > y,
                    IRelOp::Gt(Sx::S) => (x as $i) > (y as $i),
                    IRelOp::Le(Sx::U) => x <= y,
                    IRelOp::Le(Sx::S) => (x as $i) <= (y as $i),
                    IRelOp::Ge(Sx::U) => x >= y,
                    IRelOp::Ge(Sx::S) => (x as $i) >= (y as $i),
                }
            }
        }

        impl IUnOp {
            pub fn $n(&self, x: $u) -> Option<$u> {
                Some(match self {
                    IUnOp::Clz => x.leading_zeros(),
                    IUnOp::Ctz => x.trailing_zeros(),
                    IUnOp::PopCnt => x.count_ones(),
                } as $u)
            }
        }

        impl IBinOp {
            pub fn $n(&self, x: $u, y: $u) -> Option<$u> {
                match self {
                    IBinOp::Add => Some(x.wrapping_add(y)),
                    IBinOp::Sub => Some(x.wrapping_sub(y)),
                    IBinOp::Mul => Some(x.wrapping_mul(y)),
                    IBinOp::Div(Sx::U) => x.checked_div(y),
                    IBinOp::Div(Sx::S) => (x as $i).checked_div(y as $i).map(|z| z as $u),
                    IBinOp::Rem(Sx::U) => x.checked_rem(y),
                    IBinOp::Rem(Sx::S) if y == 0 => None,
                    IBinOp::Rem(Sx::S) => Some((x as $i).wrapping_rem(y as $i) as $u),
                    IBinOp::And => Some(x & y),
                    IBinOp::Or => Some(x | y),
                    IBinOp::Xor => Some(x ^ y),
                    IBinOp::Shl => Some(x.wrapping_shl(y as u32)),
                    IBinOp::Shr(Sx::U) => Some(x.wrapping_shr(y as u32)),
                    IBinOp::Shr(Sx::S) => Some((x as $i).wrapping_shr(y as u32) as $u),
                    IBinOp::Rotl => Some(x.rotate_left(y as u32)),
                    IBinOp::Rotr => Some(x.rotate_right(y as u32)),
                }
            }
        }

        #[cfg(feature = "float-types")]
        impl FRelOp {
            pub fn $n(&self, x: $u, y: $u) -> bool {
                let x = $f::from_bits(x);
                let y = $f::from_bits(y);
                match self {
                    FRelOp::Eq => x == y,
                    FRelOp::Ne => x != y,
                    FRelOp::Lt => x < y,
                    FRelOp::Gt => x > y,
                    FRelOp::Le => x <= y,
                    FRelOp::Ge => x >= y,
                }
            }
        }

        #[cfg(feature = "float-types")]
        impl FUnOp {
            pub fn $n(&self, x: $u) -> $u {
                let x = $f::from_bits(x);
                let z = match self {
                    FUnOp::Abs => float::$f::abs(x),
                    FUnOp::Neg => -x,
                    FUnOp::Ceil => float::$f::ceil(x),
                    FUnOp::Floor => float::$f::floor(x),
                    FUnOp::Trunc => float::$f::trunc(x),
                    FUnOp::Nearest => {
                        let round = float::$f::round(x);
                        if float::$f::abs(x - round) == 0.5 {
                            match round % 2. {
                                r if r == 1. => float::$f::floor(x),
                                r if r == -1. => float::$f::ceil(x),
                                r if r == 0. => round,
                                _ => unreachable!(),
                            }
                        } else {
                            round
                        }
                    }
                    FUnOp::Sqrt => float::$f::sqrt(x),
                };
                z.to_bits()
            }
        }

        #[cfg(feature = "float-types")]
        impl FBinOp {
            pub fn $n(&self, x_: $u, y_: $u) -> $u {
                let x = $f::from_bits(x_);
                let y = $f::from_bits(y_);
                let z = match self {
                    FBinOp::Add => x + y,
                    FBinOp::Sub => x - y,
                    FBinOp::Mul => x * y,
                    FBinOp::Div => x / y,
                    FBinOp::Min => x.minimum(y),
                    FBinOp::Max => x.maximum(y),
                    FBinOp::CopySign => {
                        const M: $u = 1 << ($u::BITS - 1);
                        return (x_ & !M) | (y_ & M);
                    }
                };
                z.to_bits()
            }
        }
    };
}
impl_op!(n32, u32, i32, f32);
impl_op!(n64, u64, i64, f64);

#[cfg(feature = "float-types")]
#[allow(non_upper_case_globals)]
mod float {
    pub mod f32 {
        pub const abs: fn(f32) -> f32 = libm::fabsf;
        pub const ceil: fn(f32) -> f32 = libm::ceilf;
        pub const floor: fn(f32) -> f32 = libm::floorf;
        pub const round: fn(f32) -> f32 = libm::roundf;
        pub const sqrt: fn(f32) -> f32 = libm::sqrtf;
        pub const trunc: fn(f32) -> f32 = libm::truncf;
    }

    pub mod f64 {
        pub const abs: fn(f64) -> f64 = libm::fabs;
        pub const ceil: fn(f64) -> f64 = libm::ceil;
        pub const floor: fn(f64) -> f64 = libm::floor;
        pub const round: fn(f64) -> f64 = libm::round;
        pub const sqrt: fn(f64) -> f64 = libm::sqrt;
        pub const trunc: fn(f64) -> f64 = libm::trunc;
    }
}

impl From<u8> for Sx {
    fn from(x: u8) -> Self {
        match x {
            0 => Sx::S,
            1 => Sx::U,
            _ => unreachable!(),
        }
    }
}

impl From<u8> for Nx {
    fn from(x: u8) -> Self {
        match x {
            0 => Nx::N32,
            1 => Nx::N64,
            _ => unreachable!(),
        }
    }
}

impl From<u8> for Bx {
    fn from(x: u8) -> Self {
        match x {
            0 => Bx::N32B8,
            1 => Bx::N32B16,
            2 => Bx::N64B8,
            3 => Bx::N64B16,
            4 => Bx::N64B32,
            _ => unreachable!(),
        }
    }
}

impl From<u8> for IRelOp {
    fn from(x: u8) -> Self {
        match x {
            0 => IRelOp::Eq,
            1 => IRelOp::Ne,
            x @ 2 ..= 3 => IRelOp::Lt((x - 2).into()),
            x @ 4 ..= 5 => IRelOp::Gt((x - 4).into()),
            x @ 6 ..= 7 => IRelOp::Le((x - 6).into()),
            x @ 8 ..= 9 => IRelOp::Ge((x - 8).into()),
            _ => unreachable!(),
        }
    }
}

impl From<u8> for IUnOp {
    fn from(x: u8) -> Self {
        match x {
            0 => IUnOp::Clz,
            1 => IUnOp::Ctz,
            2 => IUnOp::PopCnt,
            _ => unreachable!(),
        }
    }
}

impl From<u8> for IBinOp {
    fn from(x: u8) -> Self {
        match x {
            0 => IBinOp::Add,
            1 => IBinOp::Sub,
            2 => IBinOp::Mul,
            x @ 3 ..= 4 => IBinOp::Div((x - 3).into()),
            x @ 5 ..= 6 => IBinOp::Rem((x - 5).into()),
            7 => IBinOp::And,
            8 => IBinOp::Or,
            9 => IBinOp::Xor,
            10 => IBinOp::Shl,
            x @ 11 ..= 12 => IBinOp::Shr((x - 11).into()),
            13 => IBinOp::Rotl,
            14 => IBinOp::Rotr,
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "float-types")]
impl From<u8> for FRelOp {
    fn from(x: u8) -> Self {
        match x {
            0 => FRelOp::Eq,
            1 => FRelOp::Ne,
            2 => FRelOp::Lt,
            3 => FRelOp::Gt,
            4 => FRelOp::Le,
            5 => FRelOp::Ge,
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "float-types")]
impl From<u8> for FUnOp {
    fn from(x: u8) -> Self {
        match x {
            0 => FUnOp::Abs,
            1 => FUnOp::Neg,
            2 => FUnOp::Ceil,
            3 => FUnOp::Floor,
            4 => FUnOp::Trunc,
            5 => FUnOp::Nearest,
            6 => FUnOp::Sqrt,
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "float-types")]
impl From<u8> for FBinOp {
    fn from(x: u8) -> Self {
        match x {
            0 => FBinOp::Add,
            1 => FBinOp::Sub,
            2 => FBinOp::Mul,
            3 => FBinOp::Div,
            4 => FBinOp::Min,
            5 => FBinOp::Max,
            6 => FBinOp::CopySign,
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "threads")]
impl From<u8> for AtomicUnOp {
    fn from(x: u8) -> Self {
        match x {
            0 => AtomicUnOp::Add,
            1 => AtomicUnOp::Sub,
            2 => AtomicUnOp::And,
            3 => AtomicUnOp::Or,
            4 => AtomicUnOp::Xor,
            5 => AtomicUnOp::Xchg,
            _ => unreachable!(),
        }
    }
}

impl SectionId {
    pub fn order(self) -> u8 {
        // DataCount is actually between Element and Code.
        match self as u8 {
            x @ 0 ..= 9 => x,
            12 => 10,
            x @ 10 ..= 11 => x + 1,
            _ => unreachable!(),
        }
    }
}

pub trait TryFromByte: Sized {
    fn try_from_byte(byte: u8) -> Option<Self>;
}

pub trait UnsafeFromByte {
    // Safety: The byte must be a valid representation.
    unsafe fn from_byte_unchecked(byte: u8) -> Self;
}

impl TryFromByte for bool {
    fn try_from_byte(byte: u8) -> Option<Self> {
        Some(match byte {
            0 => false,
            1 => true,
            _ => return None,
        })
    }
}

impl UnsafeFromByte for bool {
    unsafe fn from_byte_unchecked(byte: u8) -> Self {
        unsafe { Self::try_from_byte(byte).unwrap_unchecked() }
    }
}

macro_rules! impl_from_byte {
    ($name:ident) => {
        impl UnsafeFromByte for $name {
            unsafe fn from_byte_unchecked(byte: u8) -> Self {
                unsafe { Self::unchecked_transmute_from(byte) }
            }
        }
        impl TryFromByte for $name {
            fn try_from_byte(byte: u8) -> Option<Self> {
                Self::try_from_primitive(byte).ok()
            }
        }
    };
}
impl_from_byte!(RefType);
impl_from_byte!(ValType);
impl_from_byte!(Mut);
impl_from_byte!(SectionId);
