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

use alloc::vec;
use alloc::vec::Vec;
use core::ops::Range;

use crate::error::*;
use crate::module::Parser;
use crate::util::leb128;

pub struct SideTableView<'m> {
    pub func_idx: usize,
    pub indices: &'m [u8], // including 0 and the length of metadata_array
    pub metadata: &'m [u8],
}

impl<'m> SideTableView<'m> {
    pub fn new(binary: &'m [u8]) -> Result<Self, Error> {
        let mut parser = unsafe { Parser::new(binary) };
        let num_functions = parser.parse_u16().unwrap() as usize;
        let indices_end = 2 + (num_functions + 1) * 2;
        Ok(SideTableView {
            func_idx: 0,
            indices: &binary[2 .. indices_end],
            metadata: &binary[indices_end ..],
        })
    }

    // TODO(dev/fast-interp): Make it generic since it will be used in both `Check` and `Use` modes.
    // (Returns `MResult<Metadata<'m>, M>` instead.)
    pub fn metadata(&self, func_idx: usize) -> Metadata<'m> {
        Metadata(
            &self.metadata[parse_u16(self.indices, func_idx * 2) as usize
                .. parse_u16(self.indices, (func_idx + 1) * 2) as usize],
        )
    }
}

#[derive(Default, Copy, Clone)]
pub struct Metadata<'m>(&'m [u8]);

impl<'m> Metadata<'m> {
    pub fn type_idx(&self) -> usize {
        parse_u16(self.0, 0) as usize
    }

    #[allow(dead_code)]
    pub fn parser(&self, module: &'m [u8]) -> Parser<'m> {
        unsafe { Parser::new(&module[self.parser_range()]) }
    }

    pub fn branch_table(&self) -> &[BranchTableEntry] {
        let entry_size = size_of::<BranchTableEntry>();
        assert_eq!(
            (self.0.len() - 10) % entry_size,
            0,
            "Metadata length for branch table must be divisible by {} bytes",
            entry_size
        );
        unsafe {
            core::slice::from_raw_parts(
                self.0[10 ..].as_ptr() as *const BranchTableEntry,
                self.0.len() / entry_size,
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
    let mut res = vec![];
    leb128(side_table.len(), &mut res);
    let mut index = 0;
    res.extend_from_slice(&(index as u16).to_le_bytes());
    for entry in side_table {
        index = u16::try_from(index + 10 + 6 * entry.branch_table.len()).map_err(|_| {
            #[cfg(feature = "debug")]
            eprintln!("index of MetadataEntry overflow");
            unsupported(if_debug!(Unsupported::SideTable))
        })? as usize;
        res.extend_from_slice(&(index as u16).to_le_bytes());
    }
    for entry in side_table {
        let type_idx = u16::try_from(entry.type_idx).map_err(|_| {
            #[cfg(feature = "debug")]
            eprintln!("MetadataEntry::type_idx overflow");
            unsupported(if_debug!(Unsupported::SideTable))
        })?;
        res.extend_from_slice(&type_idx.to_le_bytes());
        let range_start = u32::try_from(entry.parser_range.start).map_err(|_| {
            #[cfg(feature = "debug")]
            eprintln!("MetadataEntry::parser_range start overflow");
            unsupported(if_debug!(Unsupported::SideTable))
        })?;
        res.extend_from_slice(&range_start.to_le_bytes());
        let range_end = u32::try_from(entry.parser_range.end).map_err(|_| {
            #[cfg(feature = "debug")]
            eprintln!("MetadataEntry::parser_range end overflow");
            unsupported(if_debug!(Unsupported::SideTable))
        })?;
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
