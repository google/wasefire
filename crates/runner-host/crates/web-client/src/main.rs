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

mod app;
mod board;
mod board_components;
mod console;
mod hooks;

use cfg_if::cfg_if;

fn main() {
    wasm_logger::init(wasm_logger::Config::new(get_log_level()));

    yew::Renderer::<app::App>::new().render();
}

fn get_log_level() -> log::Level {
    cfg_if! {
        if #[cfg(feature = "web_client_log_level_trace")] {
            log::Level::Trace
        }
        else if #[cfg(feature = "web_client_log_level_debug")] {
            log::Level::Debug
        } else if #[cfg(feature = "web_client_log_level_info")] {
            log::Level::Info
        } else if #[cfg(feature = "web_client_log_level_warn")] {
            log::Level::Warn
        } else {
            log::Level::Error
        }
    }
}
