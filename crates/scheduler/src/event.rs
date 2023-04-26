// Copyright 2022 Google LLC
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
use core::borrow::Borrow;
use core::cmp::Ordering;

use wasefire_board_api::{Api as Board, Event};
use wasefire_interpreter::InstId;
use wasefire_logger as logger;

use crate::Scheduler;

pub mod button;
pub mod timer;
pub mod usb;

// TODO: This could be encoded into a u32 for performance/footprint.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Button(button::Key),
    Timer(timer::Key),
    Usb(usb::Key),
}

impl<'a> From<&'a Event> for Key {
    fn from(event: &'a Event) -> Self {
        match event {
            Event::Button(event) => Key::Button(event.into()),
            Event::Timer(event) => Key::Timer(event.into()),
            Event::Usb(event) => Key::Usb(event.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Handler {
    pub key: Key,
    pub inst: InstId,
    pub func: u32,
    pub data: u32,
}

impl PartialEq for Handler {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Equal)
    }
}

impl Eq for Handler {}

impl PartialOrd for Handler {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Handler {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl Borrow<Key> for Handler {
    fn borrow(&self) -> &Key {
        &self.key
    }
}

pub fn process<B: Board>(scheduler: &mut Scheduler<B>, event: Event) {
    let Handler { inst, func, data, .. } = match scheduler.applet.get(Key::from(&event)) {
        Some(x) => x,
        None => {
            // This should not happen because we remove pending events when disabling an event.
            logger::error!("Missing handler for event.");
            return;
        }
    };
    let mut params = vec![*func, *data];
    match event {
        Event::Button(event) => button::process(event, &mut params),
        Event::Timer(_) => timer::process(),
        Event::Usb(event) => usb::process(event),
    }
    let name = match params.len() - 2 {
        0 => "cb0",
        1 => "cb1",
        _ => unimplemented!(),
    };
    scheduler.call(*inst, name, &params);
}
