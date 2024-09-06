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

#![feature(never_type)]
#![feature(try_blocks)]

use std::path::Path;
use std::sync::Mutex;

use anyhow::Result;
use board::Board;
use clap::Parser;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{channel, Receiver};
use wasefire_board_api::Event;
#[cfg(feature = "wasm")]
use wasefire_interpreter as _;
use wasefire_scheduler::Scheduler;
use wasefire_store::{FileOptions, FileStorage};

mod board;

static STATE: Mutex<Option<board::State>> = Mutex::new(None);
static RECEIVER: Mutex<Option<Receiver<Event<Board>>>> = Mutex::new(None);

fn with_state<R>(f: impl FnOnce(&mut board::State) -> R) -> R {
    f(STATE.lock().unwrap().as_mut().unwrap())
}

#[derive(Parser)]
struct Flags {
    #[cfg(feature = "web")]
    #[clap(flatten)]
    web_options: WebOptions,
}

#[test]
fn flags() {
    <Flags as clap::CommandFactory>::command().debug_assert();
}

#[derive(clap::Args)]
struct WebOptions {
    /// Host to start the webserver.
    #[clap(long, default_value = "127.0.0.1")]
    web_host: String,

    /// Port to start the webserver.
    #[clap(long, default_value = "5000")]
    web_port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    #[cfg_attr(not(feature = "web"), allow(unused_variables))]
    let flags = Flags::parse();
    // TODO: Should be a flag controlled by xtask (value is duplicated there).
    const STORAGE: &str = "../../target/wasefire/storage.bin";
    let options = FileOptions { word_size: 4, page_size: 4096, num_pages: 16 };
    let storage = Some(FileStorage::new(Path::new(STORAGE), options).unwrap());
    let (sender, receiver) = channel(10);
    *RECEIVER.lock().unwrap() = Some(receiver);
    #[cfg(feature = "web")]
    let web = {
        let (sender, mut receiver) = channel(10);
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match event {
                    web_server::Event::Button { pressed } => {
                        with_state(|state| board::button::event(state, Some(pressed)));
                    }
                }
            }
        });
        let mut trunk = std::process::Command::new("../../scripts/wrapper.sh");
        trunk.args(["trunk", "build", "--locked", "--release", "--features=web_client_log_level_warn", "crates/web-client/index.html"]);
        wasefire_cli_tools::cmd::execute(&mut trunk)?;
        let url = format!("{}:{}", flags.web_options.web_host, flags.web_options.web_port);
        web_server::Client::new(&url, sender).await?
    };
    *STATE.lock().unwrap() = Some(board::State {
        sender,
        button: false,
        led: false,
        timers: board::timer::Timers::default(),
        uarts: board::uart::Uarts::new(),
        #[cfg(feature = "usb")]
        usb: board::usb::Usb::default(),
        storage,
        #[cfg(feature = "web")]
        web,
    });
    board::uart::Uarts::init();
    #[cfg(feature = "usb")]
    board::usb::Usb::init()?;
    #[cfg(not(feature = "web"))]
    tokio::task::spawn_blocking(|| {
        use std::io::BufRead;
        // The tokio::io::Stdin documentation recommends to use blocking IO in a dedicated thread.
        // Note that because of this, the runtime may not exit until the user press enter.
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
    });
    println!("Board initialized. Starting scheduler.");
    #[cfg(feature = "wasm")]
    const WASM: &[u8] = include_bytes!("../../../target/wasefire/applet.wasm");
    #[cfg(feature = "wasm")]
    Handle::current().spawn_blocking(|| Scheduler::<board::Board>::run(WASM)).await?;
    #[cfg(feature = "native")]
    Handle::current().spawn_blocking(|| Scheduler::<board::Board>::run()).await?;
    Ok(())
}
