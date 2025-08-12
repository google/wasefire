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

//! Board implementation for the syscall test.

#![no_std]

extern crate alloc;

use alloc::vec::Vec;

use wasefire_board_api::applet::{Handlers, Memory};
use wasefire_board_api::{Error, Failure};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Foo,
    Bar { data: u32 },
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Foo,
    Bar,
}

pub fn key(event: &Event) -> Key {
    match event {
        Event::Foo => Key::Foo,
        Event::Bar { .. } => Key::Bar,
    }
}

pub fn syscall<K: From<Key>>(
    mut memory: impl Memory, mut handlers: impl Handlers<K>, mut push: impl FnMut(Event), x2: u32,
    x3: u32, x4: u32,
) -> Result<u32, Failure> {
    match (x2, x3, x4) {
        (0, 0, x) => Ok(Error::decode(x as i32)?),
        (1, n, p) => Ok(memory.get(p, n)?.iter().map(|&x| x as u32).sum()),
        (2, n, p) => {
            let xs = memory.get_mut(p, n)?;
            (0 .. n).rev().zip(xs.iter_mut()).for_each(|(i, x)| *x = i as u8);
            Ok(n)
        }
        (3, n, 0) => {
            let p = memory.alloc(n, 1)?;
            memory.get_mut(p, n)?.iter_mut().for_each(|x| *x = n as u8);
            Ok(p)
        }
        (4 | 5, func, data) => {
            handlers.register(Key::from(x2 & 1 == 1).into(), func, data)?;
            Ok(0)
        }
        (6, key @ (0 | 1), 0) => {
            handlers.unregister(Key::from(key == 1).into())?;
            Ok(0)
        }
        (7, 0, 0) => {
            push(Event::Foo);
            Ok(0)
        }
        (7, 1, data) => {
            push(Event::Bar { data });
            Ok(0)
        }
        _ => Err(Failure::TRAP),
    }
}

pub fn callback<K: From<Key>>(
    _: impl Memory, mut handlers: impl Handlers<K>, event: Event, params: &mut Vec<u32>,
) {
    match event {
        Event::Foo => drop(handlers.unregister(Key::Foo.into())),
        Event::Bar { data } => params.push(data),
    }
}

pub fn disable(_: Key) -> Result<(), Error> {
    Ok(())
}

impl From<bool> for Key {
    fn from(value: bool) -> Self {
        match value {
            false => Key::Foo,
            true => Key::Bar,
        }
    }
}
