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

use alloc::vec::Vec;
use core::sync::atomic::Ordering;

use bytemuck::Zeroable as _;
use embedded_hal::digital::{InputPin, OutputPin as _};
use embedded_hal::spi::SpiBus as _;
use nrf52840_hal::gpio::{Floating, Input, Output, Pin, PushPull};
use nrf52840_hal::gpiote::Gpiote;
use nrf52840_hal::pac::SPI0;
use nrf52840_hal::spi::Spi;
use wasefire_board_api::Supported;
use wasefire_board_api::fingerprint::{Api, matcher, sensor};
use wasefire_error::{Code, Error};
use wasefire_logger as log;
use wasefire_sync::AtomicBool;

use crate::board::button::{Channel, Channels, channel};
use crate::with_state;

pub enum Impl {}

pub struct State {
    spi: Spi<SPI0>,
    cs: Pin<Output<PushPull>>,
    irq: Pin<Input<Floating>>,
    result: OpResult,
}

impl State {
    pub fn new(
        spi: Spi<SPI0>, cs: Pin<Output<PushPull>>, gpiote: &Gpiote, channels: &mut Channels,
        irq: Pin<Input<Floating>>,
    ) -> Self {
        let id = channels.allocate(Channel::Fpc2534).unwrap();
        channel(gpiote, id).input_pin(&irq).lo_to_hi().enable_interrupt();
        State { spi, cs, irq, result: OpResult::Empty }
    }
}

pub fn interrupt(state: &mut crate::State) {
    while state.fpc2534.irq.is_high().unwrap() {
        match handle_frame(state) {
            Ok(()) => (),
            Err(e) => {
                log::error!("Failed to handle frame: {}", e);
                break;
            }
        }
    }
}

#[cfg(feature = "test-vendor")]
pub fn vendor(frame: Vec<u8>) {
    FrameBuilder(frame).send()
}

impl Supported for Impl {}

impl Api for Impl {
    type Matcher = Self;
    type Sensor = Self;
}

impl matcher::Api for Impl {
    const TEMPLATE_ID_SIZE: usize = 2;

    fn start_enroll() -> Result<(), Error> {
        OpResult::set_state(OpResult::Enroll(None))?;
        let mut frame = FrameBuilder::new(Cmd::Enroll, 4);
        frame.push(&(IdType::GenerateNew as u16));
        frame.push(&0u16);
        Ok(frame.send())
    }

    fn read_enroll(template_id: &mut [u8]) -> Result<(), Error> {
        Error::user(Code::InvalidLength).check(template_id.len() == 2)?;
        let id = OpResult::take_state(|x| match x {
            OpResult::Enroll(Some(id)) => Ok(id),
            x => Err(x),
        })?;
        template_id.copy_from_slice(&id.to_le_bytes());
        Ok(())
    }

    fn abort_enroll() -> Result<(), Error> {
        let enroll = OpResult::take_state(|x| match x {
            OpResult::Enroll(x) => Ok(x),
            x => Err(x),
        })?;
        abort_common(enroll.is_none())
    }

    fn start_identify(template_id: Option<&[u8]>) -> Result<(), Error> {
        let mut frame = FrameBuilder::new(Cmd::Identify, 6);
        frame.push_template(template_id)?;
        frame.push(&0u16);
        OpResult::set_state(OpResult::Identify(None))?;
        Ok(frame.send())
    }

    fn read_identify(template_id: &mut [u8]) -> Result<(), Error> {
        Error::user(Code::InvalidLength).check(template_id.len() == 2)?;
        let id = OpResult::take_state(|x| match x {
            OpResult::Identify(Some(id)) => Ok(id),
            x => Err(x),
        })?;
        template_id.copy_from_slice(&id.to_le_bytes());
        Ok(())
    }

    fn abort_identify() -> Result<(), Error> {
        let identify = OpResult::take_state(|x| match x {
            OpResult::Identify(x) => Ok(x),
            x => Err(x),
        })?;
        abort_common(identify.is_none())
    }

    fn delete_template(template_id: Option<&[u8]>) -> Result<(), Error> {
        OpResult::set_state(OpResult::Delete(None))?;
        WAIT.store(true, Ordering::Relaxed);
        let mut frame = FrameBuilder::new(Cmd::DeleteTemplate, 4);
        frame.push_template(template_id)?;
        frame.send();
        while WAIT.load(Ordering::Acquire) {}
        with_state(|state| match core::mem::take(&mut state.fpc2534.result) {
            OpResult::Delete(Some(None)) => Ok(()),
            OpResult::Delete(Some(Some(error))) => Err(error),
            _ => Err(Error::internal(Code::InvalidState)),
        })
    }

    fn list_templates() -> Result<Vec<u8>, Error> {
        OpResult::set_state(OpResult::List(None))?;
        WAIT.store(true, Ordering::Relaxed);
        FrameBuilder::new(Cmd::ListTemplates, 0).send();
        while WAIT.load(Ordering::Acquire) {}
        with_state(|state| match core::mem::take(&mut state.fpc2534.result) {
            OpResult::List(Some(x)) => x,
            _ => Err(Error::internal(Code::InvalidState)),
        })
    }
}

impl sensor::Api for Impl {
    const IMAGE_WIDTH: usize = IMAGE_WIDTH as usize;
    const IMAGE_HEIGHT: usize = IMAGE_HEIGHT as usize;

    fn start_capture() -> Result<(), Error> {
        OpResult::set_state(OpResult::Capture)?;
        Ok(FrameBuilder::new(Cmd::Capture, 0).send())
    }

    fn read_capture(image: &mut [u8]) -> Result<(), Error> {
        Error::user(Code::InvalidLength)
            .check(image.len() == Self::IMAGE_WIDTH * Self::IMAGE_HEIGHT)?;
        let data = OpResult::take_state(|x| match x {
            OpResult::CaptureDataGet(Some(x)) => Ok(x),
            x => Err(x),
        })?;
        Error::internal(Code::InvalidLength).check(image.len() == data.len())?;
        image.copy_from_slice(&data);
        Ok(())
    }

    fn abort_capture() -> Result<(), Error> {
        let abort = OpResult::take_state(|x| match x {
            OpResult::Capture => Ok(true),
            OpResult::CaptureImageData => Ok(false),
            OpResult::CaptureDataGet(x) => Ok(x.is_none()),
            x => Err(x),
        })?;
        abort_common(abort)
    }
}

const PROTOCOL_VERSION: u16 = 4;

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct FrameHdr {
    version: u16, // PROTOCOL_VERSION
    type_: u16,   // one of FrameType
    flags: u16,   // set of FrameFlag
    payload_size: u16,
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct CmdHdr {
    cmd_id: u16, // one of Cmd
    type_: u16,  // one of FrameType (same as frame)
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy)]
enum FrameType {
    CmdRequest = 0x11,
    CmdResponse = 0x12,
    CmdEvent = 0x13,
}

#[derive(Clone, Copy)]
enum FrameFlag {
    SenderHost = 0x0010,
    SenderFwApp = 0x0040,
}

#[derive(Clone, Copy)]
enum Cmd {
    Status = 0x0040,
    Capture = 0x0050,
    Abort = 0x0052,
    ImageData = 0x0053,
    Enroll = 0x0054,
    Identify = 0x0055,
    ListTemplates = 0x0060,
    DeleteTemplate = 0x0061,
    DataGet = 0x0101,
}

#[derive(Clone, Copy)]
enum IdType {
    All = 0x2023,
    Specified = 0x3034,
    GenerateNew = 0x4045,
}

#[derive(Clone, Copy)]
enum StatusEvent {
    CmdFailed = 6,
}

#[derive(Clone, Copy)]
enum SystemState {
    ImageAvailable = 0x0010,
}

#[derive(Default)]
enum OpResult {
    #[default]
    Empty,
    Capture,
    CaptureImageData,
    CaptureDataGet(Option<Vec<u8>>),
    Abort(Option<Option<Error>>),
    Enroll(Option<u16>),
    Identify(Option<u16>),
    Delete(Option<Option<Error>>),
    List(Option<Result<Vec<u8>, Error>>),
}

impl OpResult {
    fn set(&mut self, x: Self) -> Result<(), Error> {
        Error::user(Code::InvalidState).check(matches!(self, OpResult::Empty))?;
        *self = x;
        Ok(())
    }

    fn set_state(x: Self) -> Result<(), Error> {
        with_state(|state| state.fpc2534.result.set(x))
    }

    fn take<T>(&mut self, f: impl FnOnce(Self) -> Result<T, Self>) -> Result<T, Error> {
        match f(core::mem::replace(self, OpResult::Empty)) {
            Ok(x) => Ok(x),
            Err(old) => {
                *self = old;
                Err(Error::user(Code::InvalidState))
            }
        }
    }

    fn take_state<T>(f: impl FnOnce(Self) -> Result<T, Self>) -> Result<T, Error> {
        with_state(|state| state.fpc2534.result.take(f))
    }
}

static WAIT: AtomicBool = AtomicBool::new(false);

fn with_cs_low<T>(state: &mut crate::State, f: impl FnOnce(&mut crate::State) -> T) -> T {
    state.fpc2534.cs.set_low().unwrap();
    cortex_m::asm::delay(32_000);
    let result = f(state);
    state.fpc2534.cs.set_high().unwrap();
    result
}

struct FrameBuilder(Vec<u8>);

impl FrameBuilder {
    fn new(cmd: Cmd, len: usize) -> Self {
        let mut frame = Vec::with_capacity(12 + len);
        frame.extend_from_slice(bytemuck::bytes_of(&FrameHdr {
            version: PROTOCOL_VERSION,
            type_: FrameType::CmdRequest as u16,
            flags: FrameFlag::SenderHost as u16,
            payload_size: 4 + len as u16,
        }));
        frame.extend_from_slice(bytemuck::bytes_of(&CmdHdr {
            cmd_id: cmd as u16,
            type_: FrameType::CmdRequest as u16,
        }));
        FrameBuilder(frame)
    }

    fn push<T: bytemuck::NoUninit>(&mut self, x: &T) {
        self.0.extend_from_slice(bytemuck::bytes_of(x))
    }

    fn push_template(&mut self, template_id: Option<&[u8]>) -> Result<(), Error> {
        match template_id {
            None => {
                self.push(&(IdType::All as u16));
                self.push(&0u16);
                Ok(())
            }
            Some(x) => {
                self.push(&(IdType::Specified as u16));
                let x = x.try_into().map_err(|_| Error::user(Code::InvalidLength))?;
                self.push(&u16::from_le_bytes(x));
                Ok(())
            }
        }
    }

    fn send(self) {
        with_state(|state| self.send_state(state))
    }

    fn send_state(self, state: &mut crate::State) {
        assert_eq!(self.0.len(), 8 + *bytemuck::from_bytes::<u16>(&self.0[6 .. 8]) as usize);
        log::debug!("SPI write {=[u8]:02x}", self.0);
        with_cs_low(state, |state| state.fpc2534.spi.write(&self.0).unwrap())
    }
}

struct FrameParser {
    pos: usize,
    data: Vec<u8>,
}

impl FrameParser {
    fn recv(state: &mut crate::State) -> Result<Self, Error> {
        with_cs_low(state, |state| {
            let mut header = FrameHdr::zeroed();
            let buffer = bytemuck::bytes_of_mut(&mut header);
            state.fpc2534.spi.read(buffer).unwrap();
            log::debug!("SPI recv header {=[u8]:02x}", buffer);
            Error::world(Code::InvalidArgument).check(header.version == PROTOCOL_VERSION)?;
            Error::world(Code::InvalidArgument)
                .check(header.flags == FrameFlag::SenderFwApp as u16)?;
            Error::world(Code::InvalidArgument).check(
                header.type_ == FrameType::CmdResponse as u16
                    || header.type_ == FrameType::CmdEvent as u16,
            )?;
            let mut payload = alloc::vec![0; header.payload_size as usize];
            state.fpc2534.spi.read(&mut payload).unwrap();
            log::debug!(
                "SPI recv payload {=[u8]:02x}",
                &payload[.. core::cmp::min(payload.len(), 16)]
            );
            Ok(FrameParser { pos: 0, data: payload })
        })
    }

    fn pop<T: bytemuck::AnyBitPattern>(&mut self) -> Result<T, Error> {
        let len = core::mem::size_of::<T>();
        Error::world(Code::InvalidLength).check(self.pos + len <= self.data.len())?;
        let result = *bytemuck::from_bytes(&self.data[self.pos ..][.. len]);
        self.pos += len;
        Ok(result)
    }

    fn done(self) -> Result<(), Error> {
        Error::world(Code::InvalidLength).check(self.data.len() == self.pos)
    }
}

fn abort_common(abort: bool) -> Result<(), Error> {
    if !abort {
        return Ok(());
    }
    OpResult::set_state(OpResult::Abort(None))?;
    WAIT.store(true, Ordering::Relaxed);
    FrameBuilder::new(Cmd::Abort, 0).send();
    while WAIT.load(Ordering::Acquire) {}
    with_state(|state| match core::mem::take(&mut state.fpc2534.result) {
        OpResult::Abort(Some(None)) => Ok(()),
        OpResult::Abort(Some(Some(error))) => Err(error),
        _ => Err(Error::internal(Code::InvalidState)),
    })
}

fn handle_frame(state: &mut crate::State) -> Result<(), Error> {
    let mut frame = FrameParser::recv(state)?;
    match frame.pop::<u16>()? {
        x if x == Cmd::Status as u16 => handle_status(state, &mut frame)?,
        x if x == Cmd::ImageData as u16 => handle_image_data(state, &mut frame)?,
        x if x == Cmd::Enroll as u16 => handle_enroll(state, &mut frame)?,
        x if x == Cmd::Identify as u16 => handle_identify(state, &mut frame)?,
        x if x == Cmd::ListTemplates as u16 => handle_list_templates(state, &mut frame)?,
        x if x == Cmd::DataGet as u16 => handle_data_get(state, &mut frame)?,
        x => {
            log::warn!("Unknown command {:04x}", x);
            return Err(Error::internal(Code::NotImplemented));
        }
    }
    frame.done()
}

fn is_event(frame: &mut FrameParser) -> Result<bool, Error> {
    match frame.pop::<u16>()? {
        x if x == FrameType::CmdResponse as u16 => Ok(false),
        x if x == FrameType::CmdEvent as u16 => Ok(true),
        x => {
            log::warn!("Unexpected frame type {:04x}", x);
            Err(Error::internal(Code::InvalidState))
        }
    }
}

fn handle_status(state: &mut crate::State, frame: &mut FrameParser) -> Result<(), Error> {
    let is_event = is_event(frame)?;
    let status_event = frame.pop::<u16>()?;
    let system_state = frame.pop::<u16>()?;
    let app_fail_code = frame.pop::<u16>()?;
    let _reserved = frame.pop::<i16>()?;
    if !is_event {
        let error = match app_fail_code {
            0 => None,
            _ => Some(Error::new(0x80, 0x8000 | app_fail_code)),
        };
        let event = match state.fpc2534.result {
            OpResult::Capture => error.map(|error| sensor::Event::CaptureError { error }.into()),
            OpResult::Abort(None) => {
                state.fpc2534.result = OpResult::Abort(Some(error));
                WAIT.store(false, Ordering::Release);
                None
            }
            OpResult::Enroll(None) => {
                error.map(|error| matcher::Event::EnrollError { error }.into())
            }
            OpResult::Identify(None) => {
                error.map(|error| matcher::Event::IdentifyError { error }.into())
            }
            OpResult::Delete(None) => {
                state.fpc2534.result = OpResult::Delete(Some(error));
                WAIT.store(false, Ordering::Release);
                None
            }
            OpResult::List(None) => {
                match error {
                    Some(error) => state.fpc2534.result = OpResult::List(Some(Err(error))),
                    None => return Err(Error::world(Code::InvalidState)),
                }
                WAIT.store(false, Ordering::Release);
                None
            }
            _ => return Err(Error::world(Code::InvalidState)),
        };
        if let Some(event) = event {
            state.fpc2534.result = OpResult::Empty;
            state.events.push(event);
        }
        return Ok(());
    }
    if status_event == StatusEvent::CmdFailed as u16 {
        let error = Error::new(0x80, 0x8000 | app_fail_code);
        let event = match state.fpc2534.result {
            OpResult::Enroll(_) => matcher::Event::EnrollError { error },
            OpResult::Identify(_) => matcher::Event::IdentifyError { error },
            _ => return Err(Error::world(Code::InvalidState)),
        };
        state.fpc2534.result = OpResult::Empty;
        state.events.push(event.into());
        return Ok(());
    }
    if system_state & SystemState::ImageAvailable as u16 != 0
        && matches!(state.fpc2534.result, OpResult::Capture)
    {
        state.fpc2534.result = OpResult::CaptureImageData;
        let mut frame = FrameBuilder::new(Cmd::ImageData, 4);
        frame.push::<u16>(&2);
        frame.push::<u16>(&0);
        frame.send_state(state);
    }
    Ok(())
}

const IMAGE_WIDTH: u16 = 96;
const IMAGE_HEIGHT: u16 = 100;

fn handle_image_data(state: &mut crate::State, frame: &mut FrameParser) -> Result<(), Error> {
    Error::world(Code::InvalidArgument).check(!is_event(frame)?)?;
    let size = frame.pop::<u32>()?;
    let width = frame.pop::<u16>()?;
    let height = frame.pop::<u16>()?;
    let type_ = frame.pop::<u16>()?;
    let max_chunk = frame.pop::<u16>()?;
    Error::world(Code::InvalidArgument).check(width == IMAGE_WIDTH)?;
    Error::world(Code::InvalidArgument).check(height == IMAGE_HEIGHT)?;
    Error::world(Code::InvalidArgument).check(size == (width * height) as u32)?;
    Error::world(Code::InvalidArgument).check(size <= max_chunk as u32)?;
    Error::world(Code::InvalidArgument).check(type_ == 2)?;
    state.fpc2534.result = OpResult::CaptureDataGet(None);
    let mut frame = FrameBuilder::new(Cmd::DataGet, 4);
    frame.push(&size);
    frame.send_state(state);
    Ok(())
}

fn handle_enroll(state: &mut crate::State, frame: &mut FrameParser) -> Result<(), Error> {
    Error::world(Code::InvalidArgument).check(is_event(frame)?)?;
    let id = frame.pop::<u16>()?;
    let feedback = frame.pop::<u8>()?;
    let remaining = frame.pop::<u8>()? as usize;
    match feedback {
        1 => {
            state.fpc2534.result = OpResult::Enroll(Some(id));
            state.events.push(matcher::Event::EnrollDone.into());
        }
        2 ..= 7 => {
            state.events.push(matcher::Event::EnrollStep { remaining }.into());
        }
        _ => return Err(Error::world(Code::InvalidArgument)),
    }
    Ok(())
}

fn handle_identify(state: &mut crate::State, frame: &mut FrameParser) -> Result<(), Error> {
    Error::world(Code::InvalidArgument).check(is_event(frame)?)?;
    let result = frame.pop::<u16>()?;
    let template_type = frame.pop::<u16>()?;
    let template_id = frame.pop::<u16>()?;
    let _tag = frame.pop::<u16>()?;
    Error::world(Code::InvalidState)
        .check(matches!(state.fpc2534.result, OpResult::Identify(None)))?;
    match result {
        0x61ec => {
            Error::world(Code::InvalidArgument).check(template_type == IdType::Specified as u16)?;
            state.fpc2534.result = OpResult::Identify(Some(template_id));
            state.events.push(matcher::Event::IdentifyDone { result: true }.into());
        }
        0xbaad => {
            state.fpc2534.result = OpResult::Empty;
            state.events.push(matcher::Event::IdentifyDone { result: false }.into());
        }
        _ => return Err(Error::world(Code::InvalidArgument)),
    }
    Ok(())
}

fn handle_list_templates(state: &mut crate::State, frame: &mut FrameParser) -> Result<(), Error> {
    Error::world(Code::InvalidArgument).check(!is_event(frame)?)?;
    let count = frame.pop::<u16>()?;
    let mut templates = Vec::with_capacity(2 * count as usize);
    for _ in 0 .. count {
        let template = frame.pop::<u16>()?;
        templates.extend_from_slice(&template.to_le_bytes());
    }
    Error::world(Code::InvalidState).check(matches!(state.fpc2534.result, OpResult::List(None)))?;
    state.fpc2534.result = OpResult::List(Some(Ok(templates)));
    WAIT.store(false, Ordering::Release);
    Ok(())
}

fn handle_data_get(state: &mut crate::State, frame: &mut FrameParser) -> Result<(), Error> {
    Error::world(Code::InvalidArgument).check(!is_event(frame)?)?;
    let remaining = frame.pop::<u32>()?;
    let data_size = frame.pop::<u32>()?;
    let mut data = Vec::with_capacity(data_size as usize);
    for _ in 0 .. data_size {
        data.push(frame.pop()?);
    }
    Error::world(Code::InvalidState).check(remaining == 0)?;
    Error::world(Code::InvalidState)
        .check(matches!(state.fpc2534.result, OpResult::CaptureDataGet(None)))?;
    state.fpc2534.result = OpResult::CaptureDataGet(Some(data));
    state.events.push(sensor::Event::CaptureDone.into());
    WAIT.store(false, Ordering::Release);
    Ok(())
}
