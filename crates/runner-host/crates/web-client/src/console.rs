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
use wasm_bindgen::JsCast;
use web_common::Command;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_hooks::prelude::*;

#[derive(Properties, PartialEq)]

pub struct Props {
    #[prop_or_default]
    pub id: usize,
    pub command_state: UseStateHandle<Option<Command>>,
    pub on_new_console_msg: Callback<String>,
}
#[function_component(Console)]
pub fn console(Props { id, command_state, on_new_console_msg }: &Props) -> Html {
    let history = use_list(vec![]);
    let console_ref = use_node_ref();
    let button_enabled = use_state(|| false);

    let onsubmit = Callback::from({
        let history = history.clone();
        let console_ref = console_ref.clone();
        let on_new_console_msg = on_new_console_msg.clone();
        move |e: SubmitEvent| {
            e.prevent_default();
            let input_form: HtmlInputElement =
                console_ref.get().unwrap().value_of().dyn_into().unwrap();
            let value = input_form.value();
            info!("sending console message: {value}");
            history.push(format!("[send]: {value}"));
            on_new_console_msg.emit(value);
            input_form.set_value("");
        }
    });

    use_effect_with(command_state.clone(), {
        let history = history.clone();
        let button_enabled = button_enabled.clone();
        move |command_state| {
            if let Some(command) = &**command_state {
                info!("Command: {command:?}");

                match command {
                    Command::Log { message } => {
                        history.push(format!("[recv]: {message}"));
                    }
                    Command::Disconnected => {
                        history.push("Disconnected from runner".to_string());
                        button_enabled.set(false);
                    }
                    Command::Connected => {
                        history.push("Connected to runner".to_string());
                        button_enabled.set(true);
                    }
                    _ => (),
                }
            }
            || ()
        }
    });

    html! {
        <div id={id.to_string()} class={"console"}>
            <div class="console-display">
                { for history.current().iter().rev().map(|message| html!(<div>{ message }</div>)) }
            </div>
            <form class="console-form" onsubmit={onsubmit}>
                <input ref={console_ref} type="text" id="consolein" />
                <input disabled={!*button_enabled}  type="submit" value="Send" />
            </form>
        </div>
    }
}
