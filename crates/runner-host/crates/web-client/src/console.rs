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
use web_common::Command;
use yew::prelude::*;
use yew_hooks::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub id: usize,
    pub command_state: UseStateHandle<Option<Command>>,
}

#[function_component(Console)]
pub fn console(Props { id, command_state }: &Props) -> Html {
    let history = use_list(vec![]);
    let button_enabled = use_state(|| false);

    use_effect_with(command_state.clone(), {
        let history = history.clone();
        let button_enabled = button_enabled.clone();
        move |command_state| {
            if let Some(command) = &**command_state {
                info!("Command: {command:?}");
                match command {
                    Command::Log { timestamp, message } => {
                        history.push((
                            MessageKind::AppletDebug { timestamp: timestamp.to_string() },
                            message.to_string(),
                        ));
                    }
                    Command::Start => {
                        history.push((MessageKind::AppletStatus, "Applet running".to_string()));
                    }
                    Command::Exit { status } => {
                        history.push((MessageKind::AppletStatus, status.to_string()));
                    }
                    Command::Disconnected => {
                        history.push((
                            MessageKind::PlatformStatus,
                            "Disconnected from platform".to_string(),
                        ));
                        button_enabled.set(false);
                    }
                    Command::Connected => {
                        history.push((
                            MessageKind::PlatformStatus,
                            "Connected to platform".to_string(),
                        ));
                        button_enabled.set(true);
                    }
                    _ => (),
                }
            }
            || ()
        }
    });

    html! {
        <div id={id.to_string()} class="console">
            <div class="console-display">{
                for history.current().iter().rev().map(|(kind, message)| kind.html(message))
            }</div>
        </div>
    }
}

enum MessageKind {
    AppletDebug { timestamp: String },
    AppletStatus,
    PlatformStatus,
}

impl MessageKind {
    fn html(&self, message: &str) -> Html {
        match self {
            MessageKind::AppletDebug { timestamp } => {
                html! {
                    <div>
                        <span class="console-timestamp">{ timestamp } { ": " }</span>
                        <span class="console-output">{ message }</span>
                    </div>
                }
            }
            MessageKind::AppletStatus | MessageKind::PlatformStatus => {
                html!(<div class="console-status">{ message }</div>)
            }
        }
    }
}
