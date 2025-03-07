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

use crate::error::*;
use crate::module::Parser;

pub const SECTION_NAME: &str = "wasefire-sidetable";

#[derive(Debug)]
pub struct SideTableView<'m> {
    pub indices: &'m [u8], // including 0 and the length of metadata_array
    pub metadata: &'m [u8],
}

impl<'m> SideTableView<'m> {
    // TODO(dev/fast-interp): Make it generic since it will be used in both `Check` and `Use` modes.
    // (Returns `MResult<Metadata<'m>, M>` instead.)
    pub fn metadata(&self, func_idx: usize) -> Metadata<'m> {
        Metadata(
            &self.metadata[parse_u16(self.indices, func_idx * 2) as usize
                .. parse_u16(self.indices, (func_idx + 1) * 2) as usize],
        )
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Metadata<'m>(&'m [u8]);

impl<'m> Metadata<'m> {
    pub fn type_idx(&self) -> usize {
        parse_u16(self.0, 0) as usize
    }

    #[allow(dead_code)]
    pub fn parser(&self, module: &'m [u8]) -> Parser<'m> {
        unsafe { Parser::new(&module[self.parser_range()]) }
    }

    pub fn branch_table(&self) -> &'m [BranchTableEntry] {
        let entry_size = size_of::<BranchTableEntry>();
        let branch_table = &self.0[10 ..];
        assert_eq!(branch_table.len() % entry_size, 0);
        unsafe {
            core::slice::from_raw_parts(
                branch_table.as_ptr().cast(),
                branch_table.len() / entry_size,
            )
        }
    }

    pub fn parser_range(&self) -> Range<usize> {
        parse_u32(self.0, 2) as usize .. parse_u32(self.0, 6) as usize
    }
}

#[derive(Default, Debug)]
pub struct MetadataEntry {
    pub type_idx: usize,
    pub parser_range: Range<usize>,
    pub branch_table: Vec<BranchTableEntry>,
}

pub fn serialize(side_table: &[MetadataEntry]) -> Result<Vec<u8>, Error> {
    let mut res = Vec::new();
    let num_funcs = try_from::<u16>("length of MetadataEntry", side_table.len())?;
    res.extend_from_slice(&num_funcs.to_le_bytes());
    let mut index = 0u16;
    res.extend_from_slice(&index.to_le_bytes());
    for entry in side_table {
        index = try_from::<u16>(
            "index of MetadataEntry",
            index as usize + 10 + 6 * entry.branch_table.len(),
        )?;
        res.extend_from_slice(&index.to_le_bytes());
    }
    for entry in side_table {
        let type_idx = try_from::<u16>("MetadataEntry::type_idx", entry.type_idx)?;
        res.extend_from_slice(&type_idx.to_le_bytes());
        let range_start =
            try_from::<u32>("MetadataEntry::parser_range start", entry.parser_range.start)?;
        res.extend_from_slice(&range_start.to_le_bytes());
        let range_end = try_from::<u32>("MetadataEntry::parser_range end", entry.parser_range.end)?;
        res.extend_from_slice(&range_end.to_le_bytes());
        for branch in &entry.branch_table {
            res.extend_from_slice(&branch.0);
        }
    }
    Ok(res)
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct BranchTableEntry([u8; 6]);

#[derive(Debug)]
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
        let delta_ip_bytes = (view.delta_ip as u16).to_le_bytes();
        let delta_stp_bytes = (view.delta_stp as u16).to_le_bytes();
        let pop_val_counts_bytes = (((view.pop_cnt << 4) | view.val_cnt) as u16).to_le_bytes();
        Ok(BranchTableEntry([
            delta_ip_bytes[0],
            delta_ip_bytes[1],
            delta_stp_bytes[0],
            delta_stp_bytes[1],
            pop_val_counts_bytes[0],
            pop_val_counts_bytes[1],
        ]))
    }

    pub fn view(self) -> BranchTableEntryView {
        let pop_val_counts = parse_u16(&self.0, 4);
        BranchTableEntryView {
            delta_ip: (parse_u16(&self.0, 0) as i16) as i32,
            delta_stp: (parse_u16(&self.0, 2) as i16) as i32,
            val_cnt: (pop_val_counts & 0xf) as u32,
            pop_cnt: (pop_val_counts >> 4) as u32,
        }
    }

    pub fn is_invalid(self) -> bool {
        self.0 == [0; 6]
    }

    pub fn invalid() -> Self {
        BranchTableEntry([0; 6])
    }
}

fn parse_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(data[offset ..][.. 2].try_into().unwrap())
}

fn parse_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset ..][.. 4].try_into().unwrap())
}

#[allow(unused_variables)]
fn try_from<T: TryFrom<usize>>(msg: &str, val: usize) -> Result<T, Error> {
    T::try_from(val).map_err(|_| {
        #[cfg(feature = "debug")]
        eprintln!("{msg} overflow");
        unsupported(if_debug!(Unsupported::SideTable))
    })
}
