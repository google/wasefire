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

use core::slice;

use bytemuck::cast;

use crate::bit_field::*;
use crate::error::*;
use crate::parser;
use crate::toctou::Use;

pub type Parser<'m> = parser::Parser<'m, Use>;

#[allow(dead_code)]
pub struct SideTable<'m> {
    indices: &'m [u16], // including 0 and the length of metadata_array
    metadata: &'m Metadata<'m>,
}
#[allow(dead_code)]
impl<'m> SideTable<'m> {
    fn metadata(&self, func_idx: usize) -> Metadata<'m> {
        Metadata(
            &self.metadata.0
                [self.indices[func_idx] as usize .. self.indices[func_idx + 1] as usize],
        )
    }
}

#[allow(dead_code)]
struct Metadata<'m>(&'m [u8]);

#[allow(dead_code)]
impl<'m> Metadata<'m> {
    pub fn type_idx(&self) -> usize {
        let bytes = &self.0[.. size_of::<usize>()];
        usize::from_ne_bytes(bytes.try_into().unwrap())
    }

    pub fn parser(&self) -> Parser<'m> {
        unsafe {
            Parser::new(&self.0[self.parser_size_start() + size_of::<u32>() .. self.parser_end()])
        }
    }

    fn branch_table(&self) -> &[BranchTableEntry] {
        let bytes = &self.0[self.parser_end() ..];
        let branch_table_len = bytes.len() / 8;
        unsafe {
            slice::from_raw_parts(bytes.as_ptr() as *const BranchTableEntry, branch_table_len)
        }
    }

    fn parser_size_start(&self) -> usize {
        size_of::<usize>()
    }

    fn parser_end(&self) -> usize {
        let u32_size = size_of::<u32>();
        let parser_size_bytes =
            &self.0[self.parser_size_start() .. self.parser_size_start() + u32_size];
        let parser_size = usize::from_ne_bytes(parser_size_bytes.try_into().unwrap());
        self.parser_size_start() + u32_size + parser_size
    }
}

#[derive(Default, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct BranchTableEntry([u8; 8]);

pub struct BranchTableEntryView {
    /// The amount to adjust the instruction pointer by if the branch is taken.
    pub delta_ip: i32,
    /// The amount to adjust the side-table pointer by if the branch is taken.
    pub delta_stp: i32,
    /// The number of values that will be copied if the branch is taken.
    pub val_cnt: u32,
    /// The number of values that will be popped if the branch is taken.
    pub pop_cnt: u32,
}

impl BranchTableEntry {
    const DELTA_IP_MASK: u64 = 0xffff;
    const DELTA_STP_MASK: u64 = 0xffff << 16;
    const VAL_CNT_MASK: u64 = 0xf << 32;
    const POP_CNT_MASK: u64 = 0xfff << 36;

    pub fn new(view: BranchTableEntryView) -> Result<Self, Error> {
        let mut fields = 0;
        fields |= into_signed_field(Self::DELTA_IP_MASK, view.delta_ip)?;
        fields |= into_signed_field(Self::DELTA_STP_MASK, view.delta_stp)?;
        fields |= into_field(Self::VAL_CNT_MASK, view.val_cnt)?;
        fields |= into_field(Self::POP_CNT_MASK, view.pop_cnt)?;
        Ok(BranchTableEntry(cast(fields)))
    }

    pub fn view(self) -> BranchTableEntryView {
        let entry = cast(self.0);
        let delta_ip = from_signed_field(Self::DELTA_IP_MASK, entry);
        let delta_stp = from_signed_field(Self::DELTA_STP_MASK, entry);
        let val_cnt = from_field(Self::VAL_CNT_MASK, entry);
        let pop_cnt = from_field(Self::POP_CNT_MASK, entry);
        BranchTableEntryView { delta_ip, delta_stp, val_cnt, pop_cnt }
    }

    pub fn is_invalid(self) -> bool {
        let entry: u64 = cast(self.0);
        entry == 0
    }

    pub fn invalid() -> Self {
        BranchTableEntry(cast(0u64))
    }
}
