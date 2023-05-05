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

use alloc::collections::{BTreeSet, VecDeque};

use wasefire_board_api::{self as board, Event};
use wasefire_interpreter::Store;
use wasefire_logger as log;

use crate::event::{Handler, Key};
use crate::{Memory, Trap};

#[derive(Debug)]
pub struct Applet<B: board::Types> {
    pub store: AppletStore,

    /// Pending events.
    events: VecDeque<Event>,

    /// Whether we returned from a callback.
    done: bool,

    handlers: BTreeSet<Handler>,

    pub hashes: AppletHashes<B>,
}

// We have to implement manually because derive is not able to find the correct bounds.
impl<B: board::Types> Default for Applet<B> {
    fn default() -> Self {
        Self {
            store: Default::default(),
            events: Default::default(),
            done: Default::default(),
            handlers: Default::default(),
            hashes: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct AppletStore(Store<'static>);

impl AppletStore {
    pub fn memory(&mut self) -> Memory {
        Memory::new(&mut self.0)
    }
}

/// Currently alive hash contexts.
#[derive(Debug)]
pub struct AppletHashes<B: board::Types>([Option<HashContext<B>>; 4]);

// We have to implement manually because derive is not able to find the correct bounds.
impl<B: board::Types> Default for AppletHashes<B> {
    fn default() -> Self {
        Self([None, None, None, None])
    }
}

#[derive(Debug)]
pub enum HashContext<B: board::Types> {
    Sha256(board::crypto::sha256::Context<B>),
}

impl<B: board::Types> AppletHashes<B> {
    pub fn insert(&mut self, hash: HashContext<B>) -> Result<usize, Trap> {
        let id = self.0.iter().position(|x| x.is_none()).ok_or(Trap)?;
        self.0[id] = Some(hash);
        Ok(id)
    }

    pub fn get_mut(&mut self, id: usize) -> Result<&mut HashContext<B>, Trap> {
        self.0.get_mut(id).ok_or(Trap)?.as_mut().ok_or(Trap)
    }

    pub fn take(&mut self, id: usize) -> Result<HashContext<B>, Trap> {
        self.0.get_mut(id).ok_or(Trap)?.take().ok_or(Trap)
    }
}

impl<B: board::Types> Applet<B> {
    pub fn store_mut(&mut self) -> &mut Store<'static> {
        &mut self.store.0
    }

    pub fn memory(&mut self) -> Memory {
        Memory::new(self.store_mut())
    }

    pub fn push(&mut self, event: Event) {
        const MAX_EVENTS: usize = 5;
        if !self.handlers.contains(&Key::from(&event)) {
            // This can happen after an event is disabled and the event queue of the board is
            // flushed.
            log::trace!("Discarding {}", log::Debug2Format(&event));
        } else if self.events.contains(&event) {
            log::trace!("Merging {}", log::Debug2Format(&event));
        } else if self.events.len() < MAX_EVENTS {
            log::debug!("Pushing {}", log::Debug2Format(&event));
            self.events.push_back(event);
        } else {
            log::warn!("Dropping {}", log::Debug2Format(&event));
        }
    }

    /// Returns the next event action.
    pub fn pop(&mut self) -> EventAction {
        if core::mem::replace(&mut self.done, false) {
            return EventAction::Reply;
        }
        match self.events.pop_front() {
            Some(event) => EventAction::Handle(event),
            None => EventAction::Wait,
        }
    }

    pub fn done(&mut self) {
        self.done = true;
    }

    pub fn enable(&mut self, handler: Handler) -> Result<(), Trap> {
        match self.handlers.insert(handler) {
            true => Ok(()),
            false => {
                log::warn!("Tried to overwrite existing handler");
                Err(Trap)
            }
        }
    }

    pub fn disable(&mut self, key: Key) -> Result<(), Trap> {
        self.events.retain(|x| Key::from(x) != key);
        match self.handlers.remove(&key) {
            true => Ok(()),
            false => {
                log::warn!("Tried to remove non-existing handler");
                Err(Trap)
            }
        }
    }

    pub fn get(&self, key: Key) -> Option<&Handler> {
        self.handlers.get(&key)
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }
}

/// Action when waiting for callbacks.
#[derive(Debug)]
pub enum EventAction {
    /// Should handle the event.
    Handle(Event),

    /// Should resume execution (we handled at least one event).
    Reply,

    /// Should suspend execution until an event is available.
    Wait,
}
