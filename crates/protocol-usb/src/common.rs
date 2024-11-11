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

//! Provides message fragmentation into packets.
//!
//! A packet is always 64 bytes. The first byte is a header:
//! - Bit 0x80 is set for the first packet of a message (mutually exclusive with last)
//! - Bit 0x40 is set for the last packet of a message (mutually exclusive with first)
//! - Bit 0x01 is set when the fragment has a footer
//! - Bits 0x3e are never set.
//!
//! The footer is the last byte of the packet. It contains the length of the content in bytes (a
//! number between 0 and 62). Without a footer, the content length is 63 bytes.
//!
//! The content starts at the second byte (after the header). If it is shorter than 63 bytes (i.e.
//! the packet has a footer), then the padding between the content and the footer is set to zero.
//!
//! A message always have at least 2 packets: the first packet and the last packet.

use alloc::vec::Vec;

use wasefire_logger as log;

pub const PACKET_NO_REQUEST: [u8; 64] = [0; 64];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Packet<'a> {
    order: Order,
    content: &'a [u8],
}

impl<'a> Packet<'a> {
    pub fn decode(bytes: &'a [u8; 64]) -> Option<Self> {
        let header = bytes[0];
        ensure(header & 0x3e == 0)?;
        let first = header & 0x80 != 0;
        let last = header & 0x40 != 0;
        let order = Order::new(first, last)?;
        let mut content = &bytes[1 ..];
        if header & 0x01 != 0 {
            let length = bytes[63] as usize;
            ensure((0 ..= 62).contains(&length))?;
            ensure_zero(&content[length .. 62])?;
            content = &content[.. length];
        };
        Some(Packet { order, content })
    }

    #[allow(clippy::identity_op)]
    pub fn encode(self, bytes: &mut [u8; 64]) {
        let Packet { order, content } = self;
        let has_footer = content.len() != 63;
        bytes[0] = 0;
        bytes[0] |= 0x80 * matches!(order, Order::First) as u8;
        bytes[0] |= 0x40 * matches!(order, Order::Last) as u8;
        bytes[0] |= 0x01 * has_footer as u8;
        bytes[1 ..][.. content.len()].copy_from_slice(content);
        if has_footer {
            bytes[1 + content.len() ..].fill(0);
            bytes[63] = content.len() as u8;
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Order {
    First,
    Middle,
    Last,
}

impl Order {
    fn new(first: bool, last: bool) -> Option<Self> {
        Some(match (first, last) {
            (true, true) => return None,
            (true, false) => Order::First,
            (false, true) => Order::Last,
            (false, false) => Order::Middle,
        })
    }
}

#[derive(Default)]
pub struct Encoder<'a> {
    message: &'a [u8],
    count: usize, // number of emitted fragments
}

impl<'a> Encoder<'a> {
    pub fn new(message: &'a [u8]) -> Self {
        Encoder { message, count: 0 }
    }
}

impl Iterator for Encoder<'_> {
    type Item = [u8; 64];

    fn next(&mut self) -> Option<[u8; 64]> {
        let total = core::cmp::max(2, (self.message.len() + 62) / 63);
        ensure(self.count < total)?;
        let first = self.count == 0;
        let start = core::cmp::min(self.message.len(), 63 * self.count);
        let length = core::cmp::min(63, self.message.len().saturating_sub(63 * self.count));
        let content = &self.message[start ..][.. length];
        self.count += 1;
        let last = self.count == total;
        let order = Order::new(first, last).unwrap();
        let mut packet = [0; 64];
        Packet { order, content }.encode(&mut packet);
        Some(packet)
    }
}

#[derive(Default)]
pub struct Decoder {
    // None until a first packet is pushed.
    message: Option<Vec<u8>>,
}

impl Decoder {
    pub fn push(mut self, packet: &[u8; 64]) -> Option<Result<Vec<u8>, Decoder>> {
        let Packet { order, content } = Packet::decode(packet)?;
        if order == Order::First {
            if self.message.is_some() {
                log::warn!("Discarding previous message on repeated first packet.");
            }
            self.message = Some(content.to_vec());
            return Some(Err(self));
        }
        let message = match &mut self.message {
            Some(x) => x,
            None => {
                log::debug!("Discarding until first packet.");
                return Some(Err(self));
            }
        };
        message.extend_from_slice(content);
        if order == Order::Last {
            return Some(Ok(core::mem::take(message)));
        }
        Some(Err(self))
    }
}

fn ensure_zero(xs: &[u8]) -> Option<()> {
    xs.iter().all(|&x| x == 0).then_some(())
}

fn ensure(cond: bool) -> Option<()> {
    cond.then_some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_at_least_two_packets() {
        assert_eq!(Encoder::new(&[]).count(), 2);
        assert_eq!(Encoder::new(&[0]).count(), 2);
        assert_eq!(Encoder::new(&[0; 126]).count(), 2);
        assert_eq!(Encoder::new(&[0; 127]).count(), 3);
    }

    #[test]
    fn message_round_trip() {
        let max_len = 3 * 63;
        let pattern: Vec<u8> = (1 ..= max_len as u8).collect();
        for len in 0 ..= max_len {
            let expected = &pattern[.. len];
            let mut decoder = Decoder::default();
            let mut packets = Encoder::new(expected);
            let result = loop {
                let packet = packets.next().unwrap();
                match decoder.push(&packet).unwrap() {
                    Ok(x) => break x,
                    Err(x) => decoder = x,
                }
            };
            assert_eq!(packets.next(), None);
            assert_eq!(result, expected);
        }
    }
}
