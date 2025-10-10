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

use crate::cursor::*;
use crate::error::*;
use crate::toctou::*;

pub const SECTION_NAME: &str = "wasefire-sidetable";

#[derive(Debug, Clone, Copy)]
pub struct SideTableView<'m> {
    pub indices: &'m [u8], // including 0 and the length of metadata_array
    pub metadata: &'m [u8],
}

impl<'m> SideTableView<'m> {
    pub fn metadata<M: Mode>(&self, func_idx: usize) -> MResult<Metadata<'m>, M> {
        let start_idx = M::open(|| func_idx.checked_mul(2))?;
        let end_idx = M::open(|| start_idx.checked_add(2))?;
        let start = parse_u16::<M>(self.indices, start_idx)? as usize;
        let end = parse_u16::<M>(self.indices, end_idx)? as usize;
        Ok(Metadata(M::open(|| self.metadata.get(start .. end))?))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Metadata<'m>(&'m [u8]);

impl<'m> Metadata<'m> {
    pub fn type_idx<M: Mode>(&self) -> MResult<usize, M> {
        parse_u16::<M>(self.0, 0).map(|x| x as usize)
    }

    pub fn branch_table<M: Mode>(&self) -> MResult<&'m [BranchTableEntry], M> {
        let entry_size = size_of::<BranchTableEntry>();
        let branch_table = M::open(|| self.0.get(10 ..))?;
        M::check(|| branch_table.len().is_multiple_of(entry_size))?;
        Ok(unsafe {
            core::slice::from_raw_parts(
                branch_table.as_ptr().cast(),
                branch_table.len() / entry_size,
            )
        })
    }

    pub fn parser_state<M: Mode>(&self) -> MResult<CursorState, M> {
        let start = parse_u32::<M>(self.0, 2).map(|x| x as usize)?;
        let end = parse_u32::<M>(self.0, 6).map(|x| x as usize)?;
        Ok(CursorState::from_range(start .. end))
    }
}

#[derive(Default, Debug)]
pub struct MetadataEntry {
    pub type_idx: usize,
    pub parser_state: CursorState,
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
            try_from::<u32>("MetadataEntry::parser_state start", entry.parser_state.start())?;
        res.extend_from_slice(&range_start.to_le_bytes());
        let range_end =
            try_from::<u32>("MetadataEntry::parser_state end", entry.parser_state.end())?;
        res.extend_from_slice(&range_end.to_le_bytes());
        for branch in &entry.branch_table {
            res.extend_from_slice(&branch.0);
        }
    }
    Ok(res)
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct BranchTableEntry([u8; 6]); // 48 bits

#[derive(Debug)]
pub struct BranchTableEntryView {
    /// The amount to adjust the instruction pointer by if the branch is taken.
    pub delta_ip: i32, // 19 bits
    /// The amount to adjust the side-table pointer by if the branch is taken.
    pub delta_stp: i32, // 13 bits
    /// The number of values that will be copied if the branch is taken.
    pub val_cnt: u32, // 4 bits
    /// The number of values that will be popped if the branch is taken.
    pub pop_cnt: u32, // 12 bits
}

impl BranchTableEntry {
    pub fn new(view: BranchTableEntryView) -> Result<Self, Error> {
        debug_assert!([0, -1].contains(&(view.delta_ip >> 18)), "{view:?}");
        debug_assert!([0, -1].contains(&(view.delta_stp >> 12)), "{view:?}");
        debug_assert!(view.val_cnt <= 0xf);
        debug_assert!(view.pop_cnt <= 0xfff);
        let delta_ip_bytes = view.delta_ip.to_le_bytes();
        let delta_stp_bytes = (view.delta_stp as u16).to_le_bytes();
        let pop_val_counts_bytes = (((view.pop_cnt << 4) | view.val_cnt) as u16).to_le_bytes();
        Ok(BranchTableEntry([
            delta_ip_bytes[0],
            delta_ip_bytes[1],
            delta_stp_bytes[0],
            (delta_stp_bytes[1] & 0x1f) | (delta_ip_bytes[2] << 5),
            pop_val_counts_bytes[0],
            pop_val_counts_bytes[1],
        ]))
    }

    pub fn view<M: Mode>(self) -> MResult<BranchTableEntryView, M> {
        let mut delta_ip = parse_u16::<M>(&self.0, 0)? as u32;
        let delta_stp = parse_u16::<M>(&self.0, 2)?;
        let pop_val_counts = parse_u16::<M>(&self.0, 4)?;
        delta_ip |= (delta_stp as u32 & 0xe000) << 3;
        let delta_ip = ((delta_ip << 13) as i32) >> 13;
        let delta_stp = (((delta_stp << 3) as i16) >> 3) as i32;
        Ok(BranchTableEntryView {
            delta_ip,
            delta_stp,
            val_cnt: (pop_val_counts & 0xf) as u32,
            pop_cnt: (pop_val_counts >> 4) as u32,
        })
    }

    pub fn is_invalid(self) -> bool {
        self.0 == [0; 6]
    }

    pub fn invalid() -> Self {
        BranchTableEntry([0; 6])
    }
}

fn parse_u16<M: Mode>(data: &[u8], offset: usize) -> MResult<u16, M> {
    let bytes = M::open(|| try { data.get(offset .. offset.checked_add(2)?)? })?;
    Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
}

fn parse_u32<M: Mode>(data: &[u8], offset: usize) -> MResult<u32, M> {
    let bytes = M::open(|| try { data.get(offset .. offset.checked_add(4)?)? })?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}

#[allow(unused_variables)]
fn try_from<T: TryFrom<usize>>(msg: &str, val: usize) -> Result<T, Error> {
    T::try_from(val).map_err(|_| {
        #[cfg(feature = "debug")]
        eprintln!("{msg} overflow");
        unsupported(if_debug!(Unsupported::SideTable))
    })
}
