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

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use anyhow::Result;
use clap::Parser;
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver};
use wasefire_board_api::Event;
#[cfg(feature = "wasm")]
use wasefire_interpreter as _;
use wasefire_one_of::exactly_one_of;
use wasefire_protocol_tokio::Pipe;
use wasefire_scheduler::Scheduler;
use wasefire_store::{FileOptions, FileStorage};

use crate::board::platform::protocol::State as ProtocolState;
use crate::board::Board;

mod board;
mod cleanup;
mod web;

exactly_one_of!["debug", "release"];
exactly_one_of!["native", "wasm"];

#[cfg(feature = "native")]
compile_error!("native is not supported");

static STATE: Mutex<Option<board::State>> = Mutex::new(None);
static RECEIVER: Mutex<Option<Receiver<Event<Board>>>> = Mutex::new(None);
static FLAGS: LazyLock<Flags> = LazyLock::new(Flags::parse);

fn with_state<R>(f: impl FnOnce(&mut board::State) -> R) -> R {
    f(STATE.lock().unwrap().as_mut().unwrap())
}

#[derive(Parser)]
struct Flags {
    /// Path of the directory containing the platform files.
    dir: PathBuf,

    /// Transport to listen to for the platform protocol.
    #[arg(long, default_value = "usb", env = "WASEFIRE_PROTOCOL")]
    protocol: Protocol,

    /// Socket address to bind to when --protocol=tcp (ignored otherwise).
    #[arg(long, default_value = "127.0.0.1:3457")]
    tcp_addr: SocketAddr,

    /// Socket path to bind to when --protocol=unix (ignored otherwise).
    #[arg(long, default_value = "/tmp/wasefire")]
    unix_path: PathBuf,

    /// The VID:PID to use for the USB device.
    ///
    /// A USB device is used when --protocol=usb or --usb-serial (ignored otherwise). Note that USB
    /// requires sudo.
    #[arg(long, default_value = "16c0:27dd")]
    usb_vid_pid: String,

    /// Whether to enable USB serial.
    #[arg(long)]
    usb_serial: bool,

    /// User interface to interact with the board.
    #[arg(long, default_value = "stdio")]
    interface: Interface,

    /// Socket address to bind to when --interface=web (ignored otherwise).
    #[arg(long, default_value = "127.0.0.1:5000")]
    web_addr: SocketAddr,

    /// Platform version (in hexadecimal).
    #[arg(long, default_value = option_env!("WASEFIRE_HOST_VERSION").unwrap_or_default())]
    version: Option<String>,

    /// Platform serial (in hexadecimal).
    #[arg(long, default_value = option_env!("WASEFIRE_HOST_SERIAL").unwrap_or_default())]
    serial: Option<String>,
}

#[test]
fn flags() {
    <Flags as clap::CommandFactory>::command().debug_assert();
}

#[derive(Clone, clap::ValueEnum)]
enum Protocol {
    Tcp,
    Unix,
    Usb,
}

#[derive(Clone, clap::ValueEnum)]
enum Interface {
    Stdio,
    Web,
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "debug")]
    env_logger::init();
    LazyLock::force(&FLAGS);
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
    wasefire_cli_tools::fs::create_dir_all(&FLAGS.dir).await?;
    let options = FileOptions { word_size: 4, page_size: 4096, num_pages: 16 };
    let storage = Some(FileStorage::new(&FLAGS.dir.join("storage.bin"), options)?);
    board::applet::init().await;
    let (sender, receiver) = channel(10);
    *RECEIVER.lock().unwrap() = Some(receiver);
    let web = match FLAGS.interface {
        Interface::Stdio => None,
        Interface::Web => Some(web::init().await?),
    };
    let push = {
        use wasefire_board_api::platform::protocol::Event;
        let sender = sender.clone();
        move |event: Event| drop(sender.try_send(event.into()))
    };
    let protocol = match FLAGS.protocol {
        Protocol::Tcp => ProtocolState::Pipe(Pipe::new_tcp(FLAGS.tcp_addr, push).await.unwrap()),
        Protocol::Unix => {
            let pipe = Pipe::new_unix(&FLAGS.unix_path, push).await.unwrap();
            cleanup::push(Box::new(move || drop(std::fs::remove_file(&FLAGS.unix_path))));
            ProtocolState::Pipe(pipe)
        }
        Protocol::Usb => ProtocolState::Usb,
    };
    let usb = board::usb::State::new(
        &FLAGS.usb_vid_pid,
        matches!(protocol, ProtocolState::Usb),
        FLAGS.usb_serial,
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
        web,
    });
    board::uart::Uarts::init();
    board::usb::init().await?;
    if matches!(FLAGS.interface, Interface::Stdio) {
        tokio::task::spawn_blocking(|| {
            use std::io::BufRead;
            // The tokio::io::Stdin documentation recommends to use blocking IO in a dedicated
            // thread. Note that because of this, the runtime may not exit until the
            // user press enter.
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
    }
    println!("Host platform running.");
    // Not sure why Rust doesn't figure out this can't return (maybe async).
    let _: ! = tokio::task::spawn_blocking(|| Scheduler::<board::Board>::run()).await?;
}
