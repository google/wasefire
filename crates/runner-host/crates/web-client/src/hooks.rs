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

use log::{info, warn};
use web_common::{Command, Event};
use yew::prelude::*;
use yew_hooks::prelude::*;

#[derive(Clone)]
pub struct UseRunnerConnectionHandle {
    pub ws: UseWebSocketHandle,
    pub command_state: UseStateHandle<Option<Command>>,
}

impl UseRunnerConnectionHandle {
    pub fn send_console_event(&self, console_msg: String) {
        warn!("Backend processing of console input not yet implemented.");
        warn!("Ignoring message: {console_msg}");
    }
    pub fn send_board_ready(&self) {
        let command = Event::BoardReady;
        self.ws.send(serde_json::to_string(&command).unwrap());
    }
    pub fn send_event(&self, event: Event) {
        self.ws.send(serde_json::to_string(&event).unwrap());
    }
}

fn set_disconnected(command_state: UseStateHandle<Option<Command>>) {
    if let Some(command) = &*command_state {
        if *command != Command::Disconnected {
            command_state.set(Some(Command::Disconnected))
        }
    }
}
#[hook]
pub fn use_runner_connection(backend_address: String) -> UseRunnerConnectionHandle {
    let command_state = use_state(|| None);
    let ws = use_websocket_with_options(
        backend_address.clone(),
        UseWebSocketOptions {
            reconnect_interval: Some(1000),
            manual: Some(false),
            onmessage: Some(Box::new({
                let command_state = command_state.clone();
                move |message| {
                    info!("Message: {message}");
                    match serde_json::from_str::<Command>(message.as_str()) {
                        Ok(command) => command_state.set(Some(command)),
                        Err(err) => warn!("Error parsing message: {err}"),
                    }
                }
            })),
            onclose: Some(Box::new({
                let command_state = command_state.clone();
                move |close_event| {
                    let close_event_type = close_event.type_();
                    warn!("Socket closed {close_event_type} setting disconnect event.");
                    set_disconnected(command_state.clone());
                }
            })),
            onerror: Some(Box::new({
                let command_state = command_state.clone();
                move |event| {
                    let event_type = event.type_();
                    warn!("Socket error: {event_type} setting disconnect event.");
                    set_disconnected(command_state.clone());
                }
            })),
            ..Default::default()
        },
    );

    return UseRunnerConnectionHandle { ws, command_state };
}
