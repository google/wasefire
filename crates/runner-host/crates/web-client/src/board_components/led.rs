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

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub id: usize,
    pub lit: bool,
}

#[function_component]
pub fn LED(&Props { id, lit }: &Props) -> Html {
    return html! {
    <div class="monochrome_led" id={id.to_string()}>

    <object class="led" type="image/svg+xml"
        data="components/monochrome_led_on.svg" style={if lit { "" } else { "display: none;" }}></object>
    <object class="led" type="image/svg+xml" style={if !lit { "" } else { "display: none;" }}
                data="components/monochrome_led_off.svg"></object>

    </div>
    };
}
