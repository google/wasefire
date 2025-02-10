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

use alloc::vec::Vec;
use core::ops::Range;

use bytemuck::{Pod, Zeroable};

use crate::error::*;
use crate::module::Parser;

#[derive(Default)]
pub struct BranchTableView<'m> {
    pub metadata: Metadata<'m>,
    pub branch_idx: usize,
}

// TODO(dev/fast-interp): Change [u16] to [u8] to not rely on alignment.
pub struct SideTableView<'m> {
    pub func_idx: usize,
    pub indices: &'m [u16], // including 0 and the length of metadata_array
    pub metadata: &'m [u16],
    pub branch_table_view: BranchTableView<'m>,
}

impl<'m> SideTableView<'m> {
    pub fn new(binary: &'m [u8]) -> Result<Self, Error> {
        let num_funcs =
            bytemuck::pod_read_unaligned::<u16>(bytemuck::cast_slice(&binary[0 .. 2])) as usize;
        let indices_end = 2 + (num_funcs + 1) * 2;
        Ok(SideTableView {
            func_idx: 0,
            indices: bytemuck::cast_slice::<_, u16>(binary.get(2 .. indices_end).unwrap()),
            metadata: bytemuck::cast_slice::<_, u16>(binary.get(indices_end ..).unwrap()),
            branch_table_view: Default::default(),
        })
    }

    pub fn branch_table_view(&mut self, func_idx: usize) -> BranchTableView<'m> {
        BranchTableView { metadata: self.metadata(func_idx), ..Default::default() }
    }

    fn metadata(&self, func_idx: usize) -> Metadata<'m> {
        Metadata(
            &self.metadata[self.indices[func_idx] as usize .. self.indices[func_idx + 1] as usize],
        )
    }
}

#[derive(Default, Copy, Clone)]
pub struct Metadata<'m>(&'m [u16]);

impl<'m> Metadata<'m> {
    pub fn type_idx(&self) -> usize {
        self.0[0] as usize
    }

    #[allow(dead_code)]
    pub fn parser(&self, module: &'m [u8]) -> Parser<'m> {
        unsafe { Parser::new(&module[self.parser_range()]) }
    }

    pub fn branch_table(&self) -> &[BranchTableEntry] {
        bytemuck::cast_slice(&self.0[5 ..])
    }

    pub fn parser_range(&self) -> Range<usize> {
        self.read_u32(1) .. self.read_u32(3)
    }

    fn read_u32(&self, idx: usize) -> usize {
        bytemuck::pod_read_unaligned::<u32>(bytemuck::cast_slice(&self.0[idx .. idx + 2])) as usize
    }
}

#[derive(Default, Debug)]
pub struct MetadataEntry {
    pub type_idx: usize,
    pub parser_range: Range<usize>,
    pub branch_table: Vec<BranchTableEntry>,
}

#[derive(Copy, Clone, Debug, Pod, Zeroable)]
#[repr(transparent)]
pub struct BranchTableEntry([u16; 3]);

impl BranchTableEntry {
    pub fn as_bytes(&self) -> &[u8; size_of::<Self>()] {
        bytemuck::cast_ref(self)
    }
}

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
    pub fn new(view: BranchTableEntryView) -> Result<Self, Error> {
        debug_assert!((i16::MIN as i32 .. i16::MAX as i32).contains(&view.delta_ip));
        debug_assert!((i16::MIN as i32 .. i16::MAX as i32).contains(&view.delta_stp));
        debug_assert!(view.val_cnt <= 0xf);
        debug_assert!(view.pop_cnt <= 0xfff);
        Ok(BranchTableEntry([
            view.delta_ip as u16,
            view.delta_stp as u16,
            ((view.pop_cnt << 4) | view.val_cnt) as u16,
        ]))
    }

    pub fn view(self) -> BranchTableEntryView {
        BranchTableEntryView {
            delta_ip: (self.0[0] as i16) as i32,
            delta_stp: (self.0[1] as i16) as i32,
            val_cnt: (self.0[2] & 0xf) as u32,
            pop_cnt: (self.0[2] >> 4) as u32,
        }
    }

    pub fn is_invalid(self) -> bool {
        self.0 == [0; 3]
    }

    pub fn invalid() -> Self {
        BranchTableEntry([0; 3])
    }
}
