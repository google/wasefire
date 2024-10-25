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

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tokio::runtime::Handle;
use wasefire_board_api::applet::Api;
use wasefire_board_api::Error;
use wasefire_cli_tools::fs;
use wasefire_error::Code;
use wasefire_protocol::applet::ExitStatus;

pub enum Impl {}

impl Api for Impl {
    unsafe fn get() -> Result<&'static [u8], Error> {
        with_state(|x| x.get())
    }

    fn start(dry_run: bool) -> Result<(), Error> {
        with_state(|x| Handle::current().block_on(x.start(dry_run)))
    }

    fn write(chunk: &[u8]) -> Result<(), Error> {
        with_state(|x| x.write(chunk))
    }

    fn finish() -> Result<(), Error> {
        with_state(|x| Handle::current().block_on(x.finish()))
    }

    fn notify_start() {
        crate::with_state(|state| {
            if let Some(web) = &mut state.web {
                web.start();
            }
        })
    }

    fn notify_exit(status: ExitStatus) {
        crate::with_state(|state| {
            if let Some(web) = &mut state.web {
                web.exit(status);
            }
        })
    }
}

pub async fn init() {
    let path = crate::FLAGS.dir.join("applet.bin");
    let applet = read(&path).await;
    *STATE.lock().unwrap() = Some(State { path, applet, update: None });
}

async fn read(path: &Path) -> Option<&'static [u8]> {
    let content = fs::read(path).await.ok()?;
    Some(Box::leak(content.into_boxed_slice()))
}

fn with_state<T>(f: impl FnOnce(&mut State) -> Result<T, Error>) -> Result<T, Error> {
    let mut state = STATE.lock().map_err(|_| Error::world(Code::InvalidState))?;
    let state = state.as_mut().ok_or_else(|| Error::internal(Code::InvalidState))?;
    f(state)
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

struct State {
    path: PathBuf,
    applet: Option<&'static [u8]>, // shared Box<[u8]>
    update: Option<Update>,
}

struct Update {
    dry_run: bool,
    buffer: Vec<u8>,
}

impl State {
    fn get(&mut self) -> Result<&'static [u8], Error> {
        if self.update.is_some() {
            return Err(Error::user(Code::InvalidState));
        }
        Ok(self.applet.unwrap_or_default())
    }

    async fn start(&mut self, dry_run: bool) -> Result<(), Error> {
        self.update = Some(Update { dry_run, buffer: Vec::new() });
        if dry_run {
            return Ok(());
        }
        if let Some(applet) = std::mem::take(&mut self.applet) {
            drop(unsafe { Box::from_raw(applet as *const [u8] as *mut [u8]) });
            fs::remove_file(&self.path).await.map_err(|_| Error::world(0))?;
        }
        Ok(())
    }

    fn write(&mut self, chunk: &[u8]) -> Result<(), Error> {
        match &mut self.update {
            Some(x) => Ok(x.buffer.extend_from_slice(chunk)),
            None => Err(Error::user(Code::InvalidState)),
        }
    }

    async fn finish(&mut self) -> Result<(), Error> {
        let update = match std::mem::take(&mut self.update) {
            Some(x) => x,
            None => return Err(Error::user(Code::InvalidState)),
        };
        if update.dry_run || update.buffer.is_empty() {
            return Ok(());
        }
        fs::write(&self.path, &update.buffer).await.map_err(|_| Error::world(0))?;
        self.applet = Some(Box::leak(update.buffer.into_boxed_slice()));
        Ok(())
    }
}
