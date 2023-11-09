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

use wasefire_board_api::{self as board, Api as Board, Event};
use wasefire_logger as log;

use self::store::{Memory, Store, StoreApi};
use crate::event::{Handler, Key};
use crate::Trap;

pub mod store;

pub struct Applet<B: Board> {
    pub store: self::store::Store,

    /// Pending events.
    events: VecDeque<Event<B>>,

    /// Whether we returned from a callback.
    #[cfg(feature = "wasm")]
    done: bool,

    handlers: BTreeSet<Handler<B>>,

    pub hashes: AppletHashes<B>,
}

// We have to implement manually because derive is not able to find the correct bounds.
impl<B: Board> Default for Applet<B> {
    fn default() -> Self {
        Self {
            store: Default::default(),
            events: Default::default(),
            #[cfg(feature = "wasm")]
            done: Default::default(),
            handlers: Default::default(),
            hashes: Default::default(),
        }
    }
}

/// Currently alive hash contexts.
pub struct AppletHashes<B: Board>([Option<HashContext<B>>; 4]);

// We have to implement manually because derive is not able to find the correct bounds.
impl<B: Board> Default for AppletHashes<B> {
    fn default() -> Self {
        Self([None, None, None, None])
    }
}

pub enum HashContext<B: Board> {
    HmacSha256(board::crypto::HmacSha256<B>),
    HmacSha384(board::crypto::HmacSha384<B>),
    Sha256(board::crypto::Sha256<B>),
    Sha384(board::crypto::Sha384<B>),
}

impl<B: Board> AppletHashes<B> {
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

impl<B: Board> Applet<B> {
    pub fn store_mut(&mut self) -> &mut Store {
        &mut self.store
    }

    pub fn memory(&mut self) -> Memory {
        self.store.memory()
    }

    pub fn push(&mut self, event: Event<B>) {
        const MAX_EVENTS: usize = 5;
        #[allow(clippy::if_same_then_else)]
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
    pub fn pop(&mut self) -> EventAction<B> {
        #[cfg(feature = "wasm")]
        if core::mem::replace(&mut self.done, false) {
            return EventAction::Reply;
        }
        match self.events.pop_front() {
            Some(event) => EventAction::Handle(event),
            None => EventAction::Wait,
        }
    }

    #[cfg(feature = "wasm")]
    pub fn done(&mut self) {
        self.done = true;
    }

    pub fn enable(&mut self, handler: Handler<B>) -> Result<(), Trap> {
        match self.handlers.insert(handler) {
            true => Ok(()),
            false => {
                log::warn!("Tried to overwrite existing handler");
                Err(Trap)
            }
        }
    }

    pub fn disable(&mut self, key: Key<B>) -> Result<(), Trap> {
        self.events.retain(|x| Key::from(x) != key);
        match self.handlers.remove(&key) {
            true => Ok(()),
            false => {
                log::warn!("Tried to remove non-existing handler");
                Err(Trap)
            }
        }
    }

    pub fn get(&self, key: Key<B>) -> Option<&Handler<B>> {
        self.handlers.get(&key)
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }
}

/// Action when waiting for callbacks.
#[derive(Debug)]
pub enum EventAction<B: Board> {
    /// Should handle the event.
    Handle(Event<B>),

    /// Should resume execution (we handled at least one event).
    #[cfg(feature = "wasm")]
    Reply,

    /// Should suspend execution until an event is available.
    Wait,
}
