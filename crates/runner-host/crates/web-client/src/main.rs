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

#![feature(try_blocks)]

use wasm_bindgen::JsValue;
use web_sys::console::{info_1, warn_1};

mod app;
mod board;
mod board_components;
mod console;
mod hooks;

fn main() {
    match gloo_utils::window().location().search() {
        Ok(x) if x.is_empty() => info_1(&JsValue::from_str("logging disabled")),
        Ok(x) => match try { x.strip_prefix("?log=")?.parse().ok()? } {
            None => warn_1(&JsValue::from_str("failed to parse location search for log")),
            Some(level) => wasm_logger::init(wasm_logger::Config::new(level)),
        },
        Err(x) => warn_1(&x),
    }
    yew::Renderer::<app::App>::new().render();
}
