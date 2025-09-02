// Copyright 2023 Google LLC
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

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot};
use warp::Filter;
use warp::ws::{Message, WebSocket};
use wasefire_logger as log;
use wasefire_protocol::applet::ExitStatus;
use web_common::{ButtonState, Command, Component};

#[derive(Debug, Copy, Clone)]
pub enum Event {
    Exit,
    Button { pressed: bool },
}

pub struct Client {
    sender: mpsc::Sender<Command>,
}

impl Client {
    pub async fn new(dir: PathBuf, addr: SocketAddr, events: mpsc::Sender<Event>) -> Result<Self> {
        let (sender, mut receiver) = oneshot::channel();
        let client = Arc::new(Mutex::new(Some((sender, events))));

        let static_files = warp::fs::dir(dir);
        let ws = warp::path("board")
            .and(warp::ws())
            .and(warp::any().map(move || client.clone()))
            .map(|ws: warp::ws::Ws, client| ws.on_upgrade(|socket| handle(socket, client)));
        let routes = warp::get().and(static_files.or(ws));

        tokio::spawn(warp::serve(routes).run(addr));
        #[cfg(feature = "log")]
        let search = format!("?log={}", ::log::max_level());
        #[cfg(not(feature = "log"))]
        let search = "";
        let url = format!("http://{addr}/{search}");

        // Wait 2 seconds for a client to connect, otherwise open a browser. The client is supposed
        // to connect every second. This should ensure that at most one client is open.
        let sleep = tokio::time::sleep(Duration::from_secs(2));
        log::info!("Waiting for client to connect.");
        tokio::select! {
            _ = sleep => {
                log::info!("Opening {} in a browser.", url);
                if opener::open_browser(&url).is_err() {
                    log::error!("Could not open {} in a browser.", url);
                }
            }
            client = &mut receiver => return Ok(client?),
        }
        Ok(receiver.await?)
    }

    pub fn println(&self, timestamp: String, message: String) {
        self.send(Command::Log { timestamp, message });
    }

    pub fn set_led(&self, state: bool) {
        self.send(Command::Set { component_id: LED_ID, state });
    }

    pub fn start(&self) {
        self.send(Command::Start);
    }

    pub fn exit(&self, status: ExitStatus) {
        self.send(Command::Exit { status });
    }

    fn send(&self, command: Command) {
        if let Err(e) = self.sender.try_send(command) {
            log::warn!("Failed to send a command to the client: {:?}", e);
        }
    }
}

const BUTTON_ID: usize = 1;
const LED_ID: usize = 2;
type ClientInput = Arc<Mutex<Option<(oneshot::Sender<Client>, mpsc::Sender<Event>)>>>;

async fn handle(mut ws: WebSocket, client: ClientInput) {
    let client = client.lock().unwrap().take();
    let (client_sender, event_sender) = match client {
        Some(x) => x,
        None => {
            let _ = ws.send(to_message(&Command::Disconnected)).await;
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws.split();
    let (cmd_sender, mut cmd_receiver) = mpsc::channel(10);
    tokio::spawn(async move {
        while let Some(command) = cmd_receiver.recv().await {
            if ws_sender.send(to_message(&command)).await.is_err() {
                log::warn!("Failed to send command.");
            }
        }
        log::info!("Client was dropped. The runner should exit.");
    });
    cmd_sender.send(Command::Connected).await.unwrap();
    let board_config = Command::BoardConfig {
        components: vec![
            Component::Button { id: BUTTON_ID },
            Component::MonochromeLed { id: LED_ID },
        ],
    };
    cmd_sender.send(board_config).await.unwrap();
    let board_ready = ws_receiver.next().await.unwrap().unwrap();
    match serde_json::from_str(board_ready.to_str().unwrap()).unwrap() {
        web_common::Event::BoardReady => (),
        event => panic!("Expected BoardReady, got {event:?}"),
    }
    if client_sender.send(Client { sender: cmd_sender }).is_err() {
        panic!("Could not send client.");
    }

    while let Some(event) = ws_receiver.next().await {
        let event = match event {
            Ok(x) => x,
            Err(e) => {
                log::warn!("Failed to receive message: {:?}", e);
                continue;
            }
        };
        if event.is_close() {
            log::debug!("The client was closed.");
            continue;
        }
        let event = match event.to_str() {
            Ok(x) => x,
            Err(()) => {
                log::warn!("Message is not text: {:?}", event);
                continue;
            }
        };
        let event = match serde_json::from_str(event) {
            Ok(x) => x,
            Err(e) => {
                log::warn!("Could not parse {:?}: {:?}", event, e);
                continue;
            }
        };
        let event = match event {
            web_common::Event::Button { component_id, state } => {
                if component_id != BUTTON_ID {
                    log::warn!("Received button event with wrong component_id.");
                    continue;
                }
                Event::Button { pressed: matches!(state, ButtonState::Pressed) }
            }
            web_common::Event::BoardReady => panic!("Unexpected BoardReady event."),
        };
        if let Err(e) = event_sender.send(event).await {
            log::warn!("Failed to send {:?}: {:?}", event, e);
        }
    }
    log::info!("The client disconnected. Exiting the runner.");
    event_sender.send(Event::Exit).await.unwrap();
}

fn to_message(command: &Command) -> Message {
    Message::text(serde_json::to_string(command).unwrap())
}
