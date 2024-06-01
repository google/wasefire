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

use log::info;
use web_common::{Command, Component, Event};
use yew::prelude::*;
use yew::{function_component, html, Html, Properties};

use crate::board_components::button::Button;
use crate::board_components::led::LED;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub command_state: UseStateHandle<Option<Command>>,
    pub on_board_ready: Callback<()>,
    pub on_event: Callback<Event>,
}

#[function_component]
pub fn Board(Props { command_state, on_board_ready, on_event }: &Props) -> Html {
    let board_config = use_state(|| None);

    use_effect_with(command_state.clone(), {
        let board_config = board_config.clone();
        let on_board_ready = on_board_ready.clone();
        move |command_state| {
            if let Some(Command::BoardConfig { components }) = &**command_state {
                info!("Board config: {components:?}");
                board_config.set(Some(components.clone()));
                on_board_ready.emit(());
            }
            || ()
        }
    });

    return html! {
        <div class="board">{
            if let Some(board_config) = &*board_config {
                board_config.iter().map(|component| match component {
                    Component::Button{id} => html!(<Button id={id} on_event={on_event} />),
                    Component::MonochromeLed{id} => html! {
                        <LED id={id} command_state={command_state.clone()} />
                    }
                }).collect::<Html>()
            } else {
                html!(<div>{"Waiting for backend connection..."}</div>)
            }
        }</div>
    };
}
