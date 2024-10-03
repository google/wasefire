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

use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Result;
use clap::Parser;
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver};
use wasefire_board_api::Event;
#[cfg(feature = "wasm")]
use wasefire_interpreter as _;
use wasefire_one_of::exactly_one_of;
use wasefire_scheduler::Scheduler;
use wasefire_store::{FileOptions, FileStorage};

use crate::board::platform::protocol::State as ProtocolState;
use crate::board::Board;

mod board;
mod cleanup;

exactly_one_of!["debug", "release"];
exactly_one_of!["native", "wasm"];

static STATE: Mutex<Option<board::State>> = Mutex::new(None);
static RECEIVER: Mutex<Option<Receiver<Event<Board>>>> = Mutex::new(None);

fn with_state<R>(f: impl FnOnce(&mut board::State) -> R) -> R {
    f(STATE.lock().unwrap().as_mut().unwrap())
}

#[derive(Parser)]
struct Flags {
    /// Directory containing files representing the flash.
    #[arg(long, default_value = "../../target/wasefire")]
    flash_dir: PathBuf,

    /// Transport to listen to for the platform protocol.
    #[arg(long, default_value = "usb")]
    protocol: Protocol,

    /// Address to bind to when --protocol=tcp (ignored otherwise).
    #[arg(long, default_value = "127.0.0.1:3457")]
    tcp_addr: std::net::SocketAddr,

    /// Socket path to bind to when --protocol=unix (ignored otherwise).
    #[arg(long, default_value = "/tmp/wasefire")]
    unix_path: std::path::PathBuf,

    /// The VID:PID to use for the USB device.
    ///
    /// A USB device is used when --protocol=usb or --usb-serial (ignored otherwise). Note that USB
    /// requires sudo.
    #[arg(long, default_value = "16c0:27dd")]
    usb_vid_pid: String,

    /// Whether to enable USB serial.
    #[arg(long)]
    usb_serial: bool,

    #[cfg(feature = "web")]
    #[command(flatten)]
    web_options: WebOptions,
}

#[derive(Clone, clap::ValueEnum)]
enum Protocol {
    Tcp,
    Unix,
    Usb,
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
    let flags = Flags::parse();
    std::panic::set_hook(Box::new(|info| {
        eprintln!("{info}");
        cleanup::shutdown(1)
    }));
    tokio::spawn(async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        let signal = select! {
            _ = sigint.recv() => SignalKind::interrupt(),
            _ = sigterm.recv() => SignalKind::terminate(),
        };
        cleanup::shutdown(128 + signal.as_raw_value());
    });
    wasefire_cli_tools::fs::create_dir_all(&flags.flash_dir).await?;
    let storage = flags.flash_dir.join("storage.bin");
    let options = FileOptions { word_size: 4, page_size: 4096, num_pages: 16 };
    let storage = Some(FileStorage::new(&storage, options)?);
    board::applet::init(flags.flash_dir.join("applet.bin")).await;
    let (sender, receiver) = channel(10);
    *RECEIVER.lock().unwrap() = Some(receiver);
    #[cfg(feature = "web")]
    let web = {
        let (sender, mut receiver) = channel(10);
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match event {
                    web_server::Event::Exit => cleanup::shutdown(0),
                    web_server::Event::Button { pressed } => {
                        with_state(|state| board::button::event(state, Some(pressed)));
                    }
                }
            }
        });
        let mut trunk = tokio::process::Command::new("../../scripts/wrapper.sh");
        trunk.args(["trunk", "build", "--release", "crates/web-client/index.html"]);
        wasefire_cli_tools::cmd::execute(&mut trunk).await?;
        let url = format!("{}:{}", flags.web_options.web_host, flags.web_options.web_port);
        web_server::Client::new(&url, sender).await?
    };
    let push = {
        use wasefire_board_api::platform::protocol::Event;
        let sender = sender.clone();
        move |event: Event| drop(sender.try_send(event.into()))
    };
    let protocol = match flags.protocol {
        Protocol::Tcp => ProtocolState::Pipe(
            wasefire_protocol_tokio::Pipe::new_tcp(flags.tcp_addr, push).await.unwrap(),
        ),
        Protocol::Unix => {
            let pipe =
                wasefire_protocol_tokio::Pipe::new_unix(&flags.unix_path, push).await.unwrap();
            let unix_path = flags.unix_path.clone();
            cleanup::push(Box::new(move || drop(std::fs::remove_file(unix_path))));
            ProtocolState::Pipe(pipe)
        }
        Protocol::Usb => ProtocolState::Usb,
    };
    let usb = board::usb::State::new(
        &flags.usb_vid_pid,
        matches!(protocol, ProtocolState::Usb),
        flags.usb_serial,
    );
    *STATE.lock().unwrap() = Some(board::State {
        sender,
        button: false,
        led: false,
        timers: board::timer::Timers::default(),
        uarts: board::uart::Uarts::new(),
        protocol,
        usb,
        storage,
        #[cfg(feature = "web")]
        web,
    });
    board::uart::Uarts::init();
    board::usb::init()?;
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
    // Not sure why Rust doesn't figure out this can't return (maybe async).
    let _: ! = tokio::task::spawn_blocking(|| Scheduler::<board::Board>::run()).await?;
}
