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

use std::sync::Mutex;

use anyhow::Result;
use tokio::runtime::Handle;
use wasefire_board_api::Supported;
use wasefire_board_api::platform::update::Api;
use wasefire_cli_tools::fs;
use wasefire_error::{Code, Error};

pub enum Impl {}

impl Supported for Impl {}

impl Api for Impl {
    fn initialize(dry_run: bool) -> Result<(), Error> {
        Ok(*STATE.lock().unwrap() = Some(State { dry_run, buffer: Vec::new() }))
    }

    fn process(chunk: &[u8]) -> Result<(), Error> {
        match &mut *STATE.lock().unwrap() {
            None => Err(Error::user(Code::InvalidState)),
            Some(state) => Ok(state.buffer.extend_from_slice(chunk)),
        }
    }

    fn finalize() -> Result<(), Error> {
        match STATE.lock().unwrap().take() {
            None => Err(Error::user(Code::InvalidState)),
            Some(State { dry_run: true, .. }) => Ok(()),
            Some(State { dry_run: false, buffer }) => {
                Handle::current().block_on(write(&buffer)).map_err(|_| Error::world(0))?;
                crate::cleanup::shutdown(0)
            }
        }
    }
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

struct State {
    dry_run: bool,
    buffer: Vec<u8>,
}

async fn write(contents: &[u8]) -> Result<()> {
    let tmp = crate::FLAGS.dir.join("platform.tmp");
    let bin = crate::FLAGS.dir.join("platform.bin");
    let mut params = fs::WriteParams::new(&tmp);
    params.options().write(true).create_new(true).mode(0o777);
    fs::write(params, contents).await?;
    let web_dir = crate::FLAGS.dir.join("web");
    if fs::exists(&web_dir).await {
        fs::remove_dir_all(&web_dir).await?;
    }
    fs::rename(tmp, bin).await
}
