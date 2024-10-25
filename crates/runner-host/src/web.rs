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

use std::process::Stdio;

use anyhow::{ensure, Result};
use tokio::io::AsyncWriteExt as _;
use tokio::process::Command;
use tokio::sync::mpsc::channel;
use wasefire_cli_tools::{cmd, fs};

use crate::{board, cleanup, with_state, FLAGS};

pub async fn init() -> Result<web_server::Client> {
    let (sender, mut receiver) = channel(10);
    tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            match event {
                web_server::Event::Exit => cleanup::shutdown(130), // like SIGINT
                web_server::Event::Button { pressed } => {
                    with_state(|state| board::button::event(state, Some(pressed)));
                }
            }
        }
    });
    let web_dir = FLAGS.dir.join("web");
    if !fs::exists(&web_dir).await {
        let mut tar = Command::new("tar");
        tar.current_dir(&FLAGS.dir);
        tar.arg("xz");
        tar.stdin(Stdio::piped());
        let mut tar = cmd::spawn(&mut tar)?;
        let mut stdin = tar.stdin.take().unwrap();
        stdin.write_all(TARBALL).await?;
        drop(stdin);
        ensure!(tar.wait().await?.success(), "Extracting web-client tarball failed.");
    }
    web_server::Client::new(web_dir, FLAGS.web_addr, sender).await
}

const TARBALL: &[u8] = include_bytes!("../crates/web-client/web.tar.gz");
