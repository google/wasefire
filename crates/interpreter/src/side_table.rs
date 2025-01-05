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

use crate::bit_field::*;
use crate::error::*;

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct BranchTableEntry(u64);

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
    const DELTA_STP_MASK: u64 = 0x7fff << 16;
    const VAL_CNT_MASK: u64 = 0x3 << 31;
    const POP_CNT_MASK: u64 = 0x3 << 33;

    pub fn new(view: BranchTableEntryView) -> Result<Self, Error> {
        let mut fields = 0;
        fields |= into_signed_field(Self::DELTA_IP_MASK, view.delta_ip)?;
        fields |= into_signed_field(Self::DELTA_STP_MASK, view.delta_stp)?;
        fields |= into_field(Self::VAL_CNT_MASK, view.val_cnt)?;
        fields |= into_field(Self::POP_CNT_MASK, view.pop_cnt)?;
        Ok(BranchTableEntry(fields))
    }

    pub fn view(self) -> BranchTableEntryView {
        let delta_ip = from_signed_field(Self::DELTA_IP_MASK, self.0);
        let delta_stp = from_signed_field(Self::DELTA_STP_MASK, self.0);
        let val_cnt = from_field(Self::VAL_CNT_MASK, self.0);
        let pop_cnt = from_field(Self::POP_CNT_MASK, self.0);
        BranchTableEntryView { delta_ip, delta_stp, val_cnt, pop_cnt }
    }

    pub fn is_invalid(self) -> bool {
        self.0 == 0
    }

    pub fn invalid() -> Self {
        SideTableEntry(0)
    }
}
