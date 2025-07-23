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

use alloc::rc::Rc;
use core::cell::Cell;

use opensk_lib::api::connection::RecvStatus;
use opensk_lib::api::user_presence::{UserPresence, UserPresenceError, UserPresenceWaitResult};
use opensk_lib::ctap::status_code::Ctap2StatusCode;
use wasefire::{button, scheduling, timer, usb};

pub(crate) fn init() -> Impl {
    Impl(None)
}

pub(crate) struct Impl(Option<State>);

struct State {
    presence: Rc<Cell<bool>>,
    _button: button::Listener<ButtonHandler>,
    _blink: crate::blink::Blink,
}

impl UserPresence for Impl {
    fn check_init(&mut self) {
        let presence = Rc::new(Cell::new(false));
        let button = button::Listener::new(0, ButtonHandler::new(&presence)).unwrap();
        let blink = crate::blink::Blink::new_ms(500);
        self.0 = Some(State { presence, _button: button, _blink: blink });
    }

    fn wait_with_timeout(
        &mut self, packet: &mut [u8; 64], timeout_ms: usize,
    ) -> UserPresenceWaitResult {
        let presence = match &self.0 {
            Some(x) => &x.presence,
            None => return Err(Ctap2StatusCode::CTAP2_ERR_VENDOR_INTERNAL_ERROR),
        };
        let timeout = timer::Timeout::new_ms(timeout_ms);
        let mut listener = usb::ctap::Listener::new(usb::ctap::Event::Read);
        while !usb::ctap::read(packet).unwrap() {
            scheduling::wait_until(|| {
                presence.get() || timeout.is_over() || listener.is_notified()
            });
            if presence.get() {
                return Ok((Ok(()), RecvStatus::Timeout));
            }
            if timeout.is_over() {
                return Ok((Err(UserPresenceError::Timeout), RecvStatus::Timeout));
            }
        }
        let endpoint = opensk_lib::api::connection::UsbEndpoint::MainHid;
        Ok((Err(UserPresenceError::Timeout), RecvStatus::Received(endpoint)))
    }

    fn check_complete(&mut self) {
        self.0 = None;
    }
}

struct ButtonHandler {
    presence: Rc<Cell<bool>>,
}

impl ButtonHandler {
    fn new(presence: &Rc<Cell<bool>>) -> Self {
        ButtonHandler { presence: presence.clone() }
    }
}

impl button::Handler for ButtonHandler {
    fn event(&self, state: button::State) {
        match state {
            button::State::Released => (),
            button::State::Pressed => self.presence.set(true),
        }
    }
}
