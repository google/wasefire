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
use std::sync::Mutex;

use anyhow::Result;
use board::Board;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{channel, Receiver};
use wasefire_board_api::Event;
use wasefire_scheduler::Scheduler;
use wasefire_store::{FileOptions, FileStorage};

use crate::board::timer::Timers;

mod board;

static STATE: Mutex<Option<board::State>> = Mutex::new(None);
static RECEIVER: Mutex<Option<Receiver<Event<Board>>>> = Mutex::new(None);

fn with_state<R>(f: impl FnOnce(&mut board::State) -> R) -> R {
    f(STATE.lock().unwrap().as_mut().unwrap())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    // TODO: Should be a flag controlled by xtask (value is duplicated there).
    const STORAGE: &str = "../../target/storage.bin";
    let options = FileOptions { word_size: 4, page_size: 4096, num_pages: 16 };
    let storage = Some(FileStorage::new(Path::new(STORAGE), options).unwrap());
    let (sender, receiver) = channel(10);
    *RECEIVER.lock().unwrap() = Some(receiver);
    *STATE.lock().unwrap() = Some(board::State {
        sender,
        button: false,
        led: false,
        timers: Timers::default(),
        #[cfg(feature = "usb")]
        usb: board::usb::Usb::default(),
        storage,
    });
    #[cfg(feature = "usb")]
    board::usb::Usb::init();
    tokio::spawn({
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
                with_state(|state| board::button::event(state, pressed));
            }
        }
    });
    println!("Running.");
    const WASM: &[u8] = include_bytes!("../../../target/applet.wasm");
    Handle::current().spawn_blocking(|| Scheduler::<board::Board>::run(WASM)).await?
}
