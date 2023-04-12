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

use std::ops::DerefMut;
use std::time::Duration;

use tokio::task::JoinHandle;
use wasefire_board_api::timer::{Api, Command, Event};
use wasefire_board_api::Error;

use crate::board::Board;

impl Api for &mut Board {
    fn count(&mut self) -> usize {
        self.state.lock().unwrap().timers.0.len()
    }

    fn arm(&mut self, i: usize, command: &Command) -> Result<(), Error> {
        let mut state = self.state.lock().unwrap();
        let sender = state.sender.clone();
        let state = state.deref_mut();
        let timer = state.timers.0.get_mut(i).ok_or(Error::User)?;
        if timer.handle.is_some() {
            return Err(Error::User);
        }
        let duration = Duration::from_millis(command.duration_ms as u64);
        if command.periodic {
            timer.handle = Some(tokio::spawn(async move {
                let mut interval = tokio::time::interval(duration);
                interval.tick().await;
                loop {
                    interval.tick().await;
                    let _ = sender.try_send(Event { timer: i }.into());
                }
            }));
        } else {
            timer.handle = Some(tokio::spawn(async move {
                tokio::time::sleep(duration).await;
                let _ = sender.try_send(Event { timer: i }.into());
            }));
        }
        Ok(())
    }

    fn disarm(&mut self, i: usize) -> Result<(), Error> {
        let mut state = self.state.lock().unwrap();
        let timer = state.timers.0.get_mut(i).ok_or(Error::User)?;
        match &timer.handle {
            Some(handle) => handle.abort(),
            None => return Err(Error::User),
        }
        timer.handle = None;
        Ok(())
    }
}

#[derive(Default)]
pub struct Timers([Timer; 5]);

#[derive(Default)]
pub struct Timer {
    handle: Option<JoinHandle<()>>,
}
