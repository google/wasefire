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

use derive_where::derive_where;
use wasefire_board_api::vendor::{Api as _, Event};
use wasefire_board_api::{self as board, Api as Board};
use wasefire_error::Error;

use crate::applet::Applet;
use crate::applet::store::StoreApi as _;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive_where(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key<B: Board> {
    pub key: board::vendor::Key<B>,
}

impl<B: Board> From<Key<B>> for crate::event::Key<B> {
    fn from(key: Key<B>) -> Self {
        crate::event::Key::Vendor(key)
    }
}

impl<'a, B: Board> From<&'a Event<B>> for Key<B> {
    fn from(event: &'a Event<B>) -> Self {
        Key { key: board::Vendor::<B>::key(&event.0) }
    }
}

impl<B: Board> Key<B> {
    pub fn disable(self) -> Result<(), Error> {
        board::Vendor::<B>::disable(self.key)
    }
}

pub fn process<B: Board>(event: Event<B>, params: &mut Vec<u32>, applet: &mut Applet<B>) {
    let memory = applet.store.memory();
    let handlers = applet.events.handlers(None);
    board::Vendor::<B>::callback(memory, handlers, event.0, params)
}
