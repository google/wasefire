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

use log::info;
use web_common::{ButtonState, Event};
use yew::prelude::*;
#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub id: usize,
    pub on_event: Callback<Event>,
}

#[function_component]
pub fn Button(Props { id, on_event }: &Props) -> Html {
    let pressed = use_state(|| false);
    let onmousedown = {
        let pressed = pressed.clone();
        let on_event = on_event.clone();
        let id = id.clone();
        Callback::from(move |_| {
            info!("Pressed");
            pressed.set(true);
            on_event.emit(Event::Button { component_id: id, state: ButtonState::Pressed });
        })
    };

    let unpress = {
        let pressed = pressed.clone();
        let on_event = on_event.clone();
        let id = id.clone();
        Callback::from(move |_| {
            info!("Unpressed");
            if *pressed {
                pressed.set(false);
                on_event.emit(Event::Button { component_id: id, state: ButtonState::Released });
            }
        })
    };

    return html! {
        <div id={id.to_string()} onmousedown={onmousedown} onmouseup={unpress.clone()} onmouseleave={unpress.clone()} class= {"button"}  >
            <img   draggable={"false"}  type="image/svg+xml" src="components/button_pressed.svg" style={if *pressed { "" } else { "display: none;" }}
            />
            <img  draggable={"false"}   type="image/svg+xml" src="components/button_unpressed.svg" style={if !*pressed { "" } else { "display: none;" }}
            />
        </div>

    };
}
