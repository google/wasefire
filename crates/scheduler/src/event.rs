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

use derivative::Derivative;
use wasefire_board_api::{Api as Board, Event};
use wasefire_interpreter::InstId;
use wasefire_logger as logger;

use crate::Scheduler;

pub mod button;
pub mod timer;
pub mod usb;

// TODO: This could be encoded into a u32 for performance/footprint.
#[derive(Derivative)]
#[derivative(Debug(bound = ""), Copy(bound = ""), Hash(bound = ""))]
#[derivative(PartialEq(bound = ""), Eq(bound = ""), Ord(bound = ""))]
#[derivative(Ord = "feature_allow_slow_enum")]
pub enum Key<B: Board> {
    Button(button::Key<B>),
    Timer(timer::Key<B>),
    Usb(usb::Key),
}

// TODO(https://github.com/mcarton/rust-derivative/issues/112): Use Clone(bound = "") instead.
impl<B: Board> Clone for Key<B> {
    fn clone(&self) -> Self {
        *self
    }
}

// TODO(https://github.com/mcarton/rust-derivative/issues/112): Use PartialOrd(bound = "") instead.
impl<B: Board> PartialOrd for Key<B> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, B: Board> From<&'a Event<B>> for Key<B> {
    fn from(event: &'a Event<B>) -> Self {
        match event {
            Event::Button(event) => Key::Button(event.into()),
            Event::Timer(event) => Key::Timer(event.into()),
            Event::Usb(event) => Key::Usb(event.into()),
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""), Clone(bound = ""))]
#[derivative(PartialEq(bound = ""), Eq(bound = ""), Ord(bound = ""))]
#[derivative(Ord = "feature_allow_slow_enum")]
pub struct Handler<B: Board> {
    pub key: Key<B>,
    pub inst: InstId,
    pub func: u32,
    pub data: u32,
}

// TODO(https://github.com/mcarton/rust-derivative/issues/112): Use PartialOrd(bound = "") instead.
impl<B: Board> PartialOrd for Handler<B> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<B: Board> Borrow<Key<B>> for Handler<B> {
    fn borrow(&self) -> &Key<B> {
        &self.key
    }
}

pub fn process<B: Board>(scheduler: &mut Scheduler<B>, event: Event<B>) {
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
