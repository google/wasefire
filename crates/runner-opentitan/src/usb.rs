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

use core::cell::{Cell, RefCell};

use critical_section::{CriticalSection, Mutex};
use earlgrey::{PINMUX_AON, USBDEV};
use usb_device::bus::{PollResult, UsbBus};
use usb_device::endpoint::{EndpointAddress, EndpointType};
use usb_device::{UsbDirection, UsbError};
use wasefire_logger as log;

const EP_COUNT: u32 = 12;
const ID_COUNT: u32 = 32;
const BUF_BYTES: u32 = 64;
const BUF_WORDS: u32 = 16;
const _: () = assert!(BUF_WORDS * 4 == BUF_BYTES);

pub struct Usb {
    /// Bitmask of enabled IN endpoints.
    used_in: u16,

    /// Bitmask of enabled OUT endpoints.
    used_out: u16,

    /// Bitmask of free buffers.
    free_bufs: Mutex<Cell<u32>>,

    /// Received buffers.
    recv_bufs: Mutex<RefCell<[Option<ReceivedBuffer>; EP_COUNT as usize]>>,
    setup_id: Mutex<Cell<Option<u8>>>,
}

#[derive(Clone, Copy)]
struct ReceivedBuffer {
    id: u8,  // < ID_COUNT
    len: u8, // <= BUF_BYTES
}

impl Usb {
    pub fn new() -> Self {
        Usb {
            used_in: 0,
            used_out: 0,
            free_bufs: Mutex::new(Cell::new(u32::MAX)),
            recv_bufs: Mutex::new(RefCell::new([None; EP_COUNT as usize])),
            setup_id: Mutex::new(Cell::new(None)),
        }
    }
}

impl UsbBus for Usb {
    fn alloc_ep(
        &mut self, ep_dir: UsbDirection, ep_addr: Option<EndpointAddress>, ep_type: EndpointType,
        max_packet_size: u16, _interval: u8,
    ) -> Result<EndpointAddress, UsbError> {
        // This is restrictive but sufficient for our usage.
        if BUF_BYTES < max_packet_size as u32 {
            return Err(UsbError::Unsupported);
        }
        let used = match ep_dir {
            UsbDirection::In => &mut self.used_in,
            UsbDirection::Out => &mut self.used_out,
        };
        let ep = match ep_addr {
            Some(ep) => {
                if ep.direction() != ep_dir || *used & (1 << ep.index()) != 0 {
                    return Err(UsbError::InvalidEndpoint);
                }
                ep
            }
            None => {
                // We skip endpoint zero. It must be explicitly allocated.
                let ep = (*used | 1).trailing_ones();
                if EP_COUNT <= ep {
                    return Err(UsbError::EndpointOverflow);
                }
                EndpointAddress::from_parts(ep as usize, ep_dir)
            }
        };
        // This is restrictive but sufficient for our usage.
        match (ep_type, ep.index()) {
            (EndpointType::Control, 0) => (),
            (EndpointType::Bulk | EndpointType::Interrupt, x) if 0 < x => (),
            _ => return Err(UsbError::Unsupported),
        }
        *used |= 1 << ep.index();
        Ok(ep)
    }

    fn enable(&mut self) {
        // TODO: Generate top_earlgrey_pinmux_{insel,peripheral_in}. This should be UsbdevSense and
        // ConstantOne.
        PINMUX_AON.mio_periph_insel(56).reset().r#in(1).reg.write();

        // Configure the physical interface.
        USBDEV.phy_config().reset().reg.write();

        // Fill the available buffer FIFOs.
        self.fill_avbuffer(Fifo::Setup);
        self.fill_avbuffer(Fifo::Out);

        // Enable reset interrupt.
        USBDEV.intr_enable().reset().link_reset(true).reg.write();

        // Enable the interface.
        USBDEV.usbctrl().reset().enable(true).reg.write();
    }

    fn reset(&self) {
        log::info!("USB reset");

        // Disable all interrupts.
        USBDEV.intr_enable().reset().reg.write();

        // Enable all allocated endpoints. This is not compliant but usb-device design.
        let mut ep_in_enable = USBDEV.ep_in_enable().reset();
        let mut ep_out_enable = USBDEV.ep_out_enable().reset();
        let mut rxenable_out = USBDEV.rxenable_out().reset();
        let mut rxenable_setup = USBDEV.rxenable_setup().reset();
        let mut set_nak_out = USBDEV.set_nak_out().reset();
        for i in 0 .. EP_COUNT {
            if self.used_in & (1 << i) != 0 {
                ep_in_enable.enable(i as u8, true);
            }
            if self.used_out & (1 << i) != 0 {
                ep_out_enable.enable(i as u8, true);
                rxenable_out.out(i as u8, true);
                set_nak_out.enable(i as u8, true);
            }
        }
        rxenable_setup.setup(0, true);
        ep_in_enable.reg.write();
        ep_out_enable.reg.write();
        rxenable_out.reg.write();
        rxenable_setup.reg.write();
        set_nak_out.reg.write();

        // Enable interrupts.
        let mut intr = USBDEV.intr_enable().reset();
        // TODO: We should probably also check USBSTAT and bad conditions like link_out_err,
        // av_empty, rx_full.
        intr.link_reset(true).pkt_sent(true).pkt_received(true);
        intr.reg.write();
    }

    fn set_device_address(&self, addr: u8) {
        USBDEV.usbctrl().modify().device_address(addr as u32).reg.write();
    }

    fn write(&self, ep_addr: EndpointAddress, src: &[u8]) -> Result<usize, UsbError> {
        if ep_addr.is_out() || !self.is_used(ep_addr) {
            return Err(UsbError::InvalidEndpoint);
        }
        if (BUF_BYTES as usize) < src.len() {
            return Err(UsbError::BufferOverflow);
        }
        critical_section::with(|cs| {
            let ep = ep_addr.index() as u32;
            let configin = USBDEV.configin(ep).read();
            if configin.rdy() {
                return Err(UsbError::WouldBlock);
            }
            if configin.pend() {
                self.clear_pending(cs, ep, configin.buffer());
            }
            let Some(id) = self.alloc_buffer(cs) else {
                return Err(UsbError::EndpointMemoryOverflow);
            };
            buffer_write(id, src);
            let mut configin = USBDEV.configin(ep).reset();
            configin.buffer(id).size(src.len() as u32).rdy(true);
            configin.reg.write();
            Ok(src.len())
        })
    }

    fn read(&self, ep_addr: EndpointAddress, dst: &mut [u8]) -> Result<usize, UsbError> {
        if ep_addr.is_in() || !self.is_used(ep_addr) {
            return Err(UsbError::InvalidEndpoint);
        }
        critical_section::with(|cs| {
            let ep = ep_addr.index();
            if ep == 0 {
                if let Some(id) = self.setup_id.borrow(cs).take() {
                    let id = id as u32;
                    let len = 8;
                    self.read_buffer(cs, id, len, dst).inspect_err(|_| {
                        self.setup_id.borrow(cs).set(Some(id as u8));
                    })?;
                    USBDEV.rxenable_out().modify().out(ep as u8, true).reg.write();
                    return Ok(len);
                }
            }
            let Some(recv) = self.recv_bufs.borrow_ref_mut(cs)[ep].take() else {
                return Err(UsbError::WouldBlock);
            };
            let id = recv.id as u32;
            let len = recv.len as usize;
            self.read_buffer(cs, id, len, dst).inspect_err(|_| {
                self.recv_bufs.borrow_ref_mut(cs)[ep] = Some(recv);
            })?;
            USBDEV.rxenable_out().modify().out(ep as u8, true).reg.write();
            Ok(len)
        })
    }

    fn set_stalled(&self, ep_addr: EndpointAddress, stalled: bool) {
        match ep_addr.direction() {
            UsbDirection::Out => {
                USBDEV.out_stall().modify().endpoint(ep_addr.index() as u8, stalled).reg.write()
            }
            UsbDirection::In => {
                USBDEV.in_stall().modify().endpoint(ep_addr.index() as u8, stalled).reg.write()
            }
        }
    }

    fn is_stalled(&self, ep_addr: EndpointAddress) -> bool {
        match ep_addr.direction() {
            UsbDirection::Out => USBDEV.out_stall().read().endpoint(ep_addr.index() as u8),
            UsbDirection::In => USBDEV.in_stall().read().endpoint(ep_addr.index() as u8),
        }
    }

    fn suspend(&self) {
        todo!()
    }

    fn resume(&self) {
        todo!()
    }

    fn poll(&self) -> PollResult {
        critical_section::with(|cs| {
            let state = USBDEV.intr_state().read();
            for ep in 0 .. EP_COUNT {
                if self.used_in & (1 << ep) == 0 {
                    continue;
                }
                let configin = USBDEV.configin(ep).read();
                if configin.pend() {
                    self.clear_pending(cs, ep, configin.buffer());
                }
            }
            if state.link_reset() {
                USBDEV.intr_state().write_raw(u32::MAX);
                return PollResult::Reset;
            }
            let mut ep_in_complete = 0;
            if state.pkt_received() {
                while !USBDEV.usbstat().read().rx_empty() {
                    let rxfifo = USBDEV.rxfifo().read();
                    if let Some(id) = self.alloc_buffer(cs) {
                        self.free_buffer(cs, id, Some(Fifo::new(rxfifo.setup())));
                    }
                    let id = rxfifo.buffer() as u8;
                    let len = rxfifo.size() as u8;
                    let ep = rxfifo.ep();
                    if rxfifo.setup() {
                        if ep != 0 {
                            log::error!("SETUP transaction on non-zero endpoint");
                        }
                        if len != 8 {
                            log::error!("SETUP transaction of {} bytes", len);
                        }
                        if self.setup_id.borrow(cs).replace(Some(id)).is_some() {
                            log::error!("nested SETUP transaction");
                        }
                    } else {
                        let recv = ReceivedBuffer { id, len };
                        if self.recv_bufs.borrow_ref_mut(cs)[ep as usize].replace(recv).is_some() {
                            log::error!("nested OUT transaction");
                        }
                    }
                }
                USBDEV.intr_state().reset().pkt_received(true).reg.write();
            }
            if state.pkt_sent() {
                let in_sent = USBDEV.in_sent().read();
                for ep in 0 .. EP_COUNT {
                    if in_sent.sent(ep as u8) {
                        ep_in_complete |= 1 << ep;
                        let id = USBDEV.configin(ep).read().buffer();
                        self.free_buffer(cs, id, None);
                    }
                }
                USBDEV.in_sent().write_raw(in_sent.reg.value());
                USBDEV.intr_state().reset().pkt_sent(true).reg.write();
            }
            let ep_setup = self.setup_id.borrow(cs).get().is_some() as u16;
            let mut ep_out = 0;
            for ep in 0 .. EP_COUNT {
                ep_out |= (self.recv_bufs.borrow_ref(cs)[ep as usize].is_some() as u16) << ep;
            }
            if ep_setup & ep_out & 1 == 1 {
                // usb-device ignores OUT ACK when a SETUP arrives right after.
                let recv = self.recv_bufs.borrow_ref_mut(cs)[0].take().unwrap();
                if recv.len == 0 {
                    log::debug!("ignoring EP 0 OUT because of SETUP");
                } else {
                    log::error!("ignoring {} bytes on EP 0 OUT", recv.len);
                }
                self.free_buffer(cs, recv.id as u32, None);
                ep_out &= !1;
            }
            if ep_out == 0 && ep_in_complete == 0 && ep_setup == 0 {
                PollResult::None
            } else {
                PollResult::Data { ep_out, ep_in_complete, ep_setup }
            }
        })
    }
}

impl Usb {
    fn is_used(&self, ep_addr: EndpointAddress) -> bool {
        let used = match ep_addr.direction() {
            UsbDirection::In => &self.used_in,
            UsbDirection::Out => &self.used_out,
        };
        let ep = ep_addr.index();
        ep < EP_COUNT as usize && used & (1 << ep) != 0
    }

    /// Acquires the next free buffer.
    fn alloc_buffer(&self, cs: CriticalSection) -> Option<u32> {
        let free = self.free_bufs.borrow(cs).get();
        if free == 0 {
            log::error!("out of free buffers");
            return None;
        }
        let id = free.trailing_zeros();
        self.free_bufs.borrow(cs).set(free & !(1 << id));
        Some(id)
    }

    /// Releases a buffer, making it free again.
    fn free_buffer(&self, cs: CriticalSection, id: u32, fifo: Option<Fifo>) {
        let free = self.free_bufs.borrow(cs).get();
        if ID_COUNT <= id || free & (1 << id) != 0 {
            log::error!("invalid free buffer: {:08x} {}", free, id);
        }
        if fifo != Some(Fifo::Out) && !Fifo::Setup.full() {
            Fifo::Setup.push(id);
        } else if fifo != Some(Fifo::Setup) && !Fifo::Out.full() {
            Fifo::Out.push(id);
        } else {
            self.free_bufs.borrow(cs).set(free | (1 << id));
        }
    }

    /// Fills the available buffer FIFO.
    fn fill_avbuffer(&self, fifo: Fifo) {
        critical_section::with(|cs| {
            while !fifo.full() {
                let Some(id) = self.alloc_buffer(cs) else { break };
                self.free_buffer(cs, id, Some(fifo));
            }
        })
    }

    /// Reads from a buffer then free it.
    fn read_buffer(
        &self, cs: CriticalSection, id: u32, len: usize, dst: &mut [u8],
    ) -> Result<(), UsbError> {
        if dst.len() < len {
            log::warn!("buffer overflow");
            return Err(UsbError::BufferOverflow);
        }
        buffer_read(id, &mut dst[.. len]);
        self.free_buffer(cs, id, None);
        Ok(())
    }

    fn clear_pending(&self, cs: CriticalSection, ep: u32, id: u32) {
        log::warn!("discarding pending IN transfer ep={} id={}", ep, id);
        self.free_buffer(cs, id, None);
        USBDEV.configin(ep).reset().pend(true).reg.write();
    }
}

fn buffer_read(id: u32, dst: &mut [u8]) {
    for (dst, src) in dst.chunks_mut(4).zip(id * BUF_WORDS ..) {
        let src = USBDEV.buffer(src).read_raw();
        dst.copy_from_slice(&src.to_ne_bytes()[.. dst.len()]);
    }
}

fn buffer_write(id: u32, src: &[u8]) {
    for (src, dst) in src.chunks(4).zip(id * BUF_WORDS ..) {
        let mut tmp = [0u8; 4];
        tmp[.. src.len()].copy_from_slice(src);
        USBDEV.buffer(dst).write_raw(u32::from_ne_bytes(tmp));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fifo {
    Setup,
    Out,
}

impl Fifo {
    fn new(setup: bool) -> Fifo {
        match setup {
            true => Fifo::Setup,
            false => Fifo::Out,
        }
    }

    fn full(self) -> bool {
        let usbstat = USBDEV.usbstat().read();
        match self {
            Fifo::Setup => usbstat.av_setup_full(),
            Fifo::Out => usbstat.av_out_full(),
        }
    }

    fn push(self, id: u32) {
        match self {
            Fifo::Setup => USBDEV.avsetupbuffer().reset().buffer(id).reg.write(),
            Fifo::Out => USBDEV.avoutbuffer().reset().buffer(id).reg.write(),
        }
    }
}
