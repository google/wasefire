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

#![feature(core_intrinsics)]
#![feature(try_blocks)]

use std::io::BufRead;
use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use tokio::runtime::Handle;
use tokio::sync::mpsc::channel;
use wasefire_scheduler::Scheduler;
use wasefire_store::{FileOptions, FileStorage};

use crate::board::timer::Timers;

mod board;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    // TODO: Should be a flag controlled by xtask (value is duplicated there).
    const STORAGE: &str = "../../target/storage.bin";
    let options = FileOptions { word_size: 4, page_size: 4096, num_pages: 16 };
    let storage = Some(FileStorage::new(Path::new(STORAGE), options).unwrap());
    let (sender, receiver) = channel(10);
    let state = Arc::new(Mutex::new(board::State {
        sender,
        button: false,
        led: false,
        timers: Timers::default(),
        #[cfg(feature = "usb")]
        usb: board::usb::Usb::default(),
        storage,
    }));
    #[cfg(feature = "usb")]
    board::usb::Usb::init(state.clone());
    tokio::spawn({
        let state = state.clone();
        async move {
            for line in std::io::stdin().lock().lines() {
                let pressed = match line.unwrap().as_str() {
                    "button" => None,
                    "press" => Some(true),
                    "release" => Some(false),
                    x => {
                        println!("Unrecognized command: {x}");
                        continue;
                    }
                };
                let mut state = state.lock().unwrap();
                board::button::event(&mut state, pressed);
            }
        }
    });
    println!("Running.");
    Handle::current().spawn_blocking(|| Scheduler::run(board::Board { receiver, state })).await?
}
