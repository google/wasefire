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

use gloo_utils::window;
use log::{info, warn};
use wasm_bindgen::UnwrapThrowExt;
use web_common::Command;
use yew::prelude::*;

use crate::board::Board;
use crate::console::Console;
use crate::hooks::use_runner_connection;

#[function_component(App)]
pub fn app() -> Html {
    let runner_connection = {
        let host = window().location().host().unwrap_throw();
        let scheme = window().location().protocol().unwrap_throw().replace("http", "ws");
        let web_socket_url = format!("{scheme}//{host}/board");
        info!("Connecting to runner at {web_socket_url}");
        use_runner_connection(web_socket_url)
    };
    let on_new_console_msg = Callback::from({
        let runner_connection = runner_connection.clone();
        move |msg| runner_connection.send_console_event(msg)
    });
    let on_board_ready = Callback::from({
        let runner_connection = runner_connection.clone();
        move |()| runner_connection.send_board_ready()
    });
    let send_event_callback = Callback::from({
        let runner_connection = runner_connection.clone();
        move |event| runner_connection.send_event(event)
    });

    use_effect_with(runner_connection.command_state.clone(), move |command_state| {
        if let Some(command) = &**command_state {
            info!("Command: {command:?}");
            match command {
                Command::Connected => info!("Connected to runner"),
                Command::Disconnected => warn!("Disconnected from runner"),
                _ => (), // Command for other component so ignoring.
            }
        }
        || ()
    });

    html! {
        <main>
            <object class="title" type="image/svg+xml" data="title.svg" />
            <Console
                id={0}
                command_state={runner_connection.command_state.clone()}
                on_new_console_msg={on_new_console_msg}
            />
            <Board
                command_state={runner_connection.command_state}
                on_board_ready={on_board_ready}
                on_event={send_event_callback}
            />
        </main>
    }
}
