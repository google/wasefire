// Copyright 2025 Google LLC
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

#![no_std]

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Advertisement {
    pub ticks: u32,
    pub freq: u16,
    pub rssi: i8,
    pub pdu_type: u8,
    pub addr: [u8; 6],
    pub data_len: u8,
    pub data: [u8; 31],
    _padding: [u8; 2],
}

impl Advertisement {
    pub fn data(&self) -> &[u8] {
        &self.data[.. self.data_len as usize]
    }
}

pub const SYSCALL_ID: u32 = 0x80000001;
pub const OP_REGISTER: u32 = 1;
pub const OP_UNREGISTER: u32 = 2;
pub const OP_READ_PACKET: u32 = 3;
