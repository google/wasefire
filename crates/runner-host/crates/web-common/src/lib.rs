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

//! Interface between the web client and web server.
//!
//! The web client plays the role of the hardware. The web server is the host runner.

use serde::{Deserialize, Serialize};
use wasefire_protocol::applet::ExitStatus;

/// Events from the hardware.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "componentType", rename_all = "snake_case")]
pub enum Event {
    #[serde(rename_all = "camelCase")]
    Button {
        component_id: usize,
        state: ButtonState,
    },

    BoardReady,
}

/// Commands for the hardware.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    /// Describes the components of the hardware.
    BoardConfig { components: Vec<Component> },

    /// Connection accepted.
    Connected,

    /// Connection rejected (a client is already connected).
    Disconnected,

    /// Sets a LED on or off.
    // TODO: Rename this to Led.
    #[serde(rename_all = "camelCase")]
    Set { component_id: usize, state: bool },

    /// Prints a debug message.
    Log { timestamp: String, message: String },

    /// Indicates that the applet started.
    Start,

    /// Indicates that the applet exited.
    Exit { status: ExitStatus },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ButtonState {
    #[serde(rename = "pressed")]
    Pressed,
    #[serde(rename = "released")]
    Released,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Component {
    MonochromeLed { id: usize },
    Button { id: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_config() {
        let board = Command::BoardConfig {
            components: vec![Component::MonochromeLed { id: 1 }, Component::Button { id: 2 }],
        };
        assert_eq!(
            serde_json::to_string(&board).unwrap(),
            "{\"type\":\"board_config\",\"components\":[{\"type\":\"monochrome_led\",\"id\":1},{\"\
             type\":\"button\",\"id\":2}]}"
        );
        assert_eq!(
            serde_json::from_str::<Command>(
                r#"{ "type": "board_config",
                     "components": [
                       { "type":"monochrome_led", "id": 1 },
                       { "type": "button", "id":2 } ] }"#
            )
            .unwrap(),
            board
        );
    }

    #[test]
    fn set() {
        let set = Command::Set { component_id: 1, state: true };
        assert_eq!(
            serde_json::to_string(&set).unwrap(),
            "{\"type\":\"set\",\"componentId\":1,\"state\":true}"
        );
        assert_eq!(
            serde_json::from_str::<Command>(
                r#"{ "type": "set", "componentId": 1, "state": true }"#
            )
            .unwrap(),
            set
        );
    }

    #[test]
    fn log() {
        let log = Command::Log { message: "hello".to_string() };
        assert_eq!(
            serde_json::to_string(&log).unwrap(),
            "{\"type\":\"log\",\"message\":\"hello\"}"
        );
        assert_eq!(
            serde_json::from_str::<Command>(r#"{ "type": "log", "message": "hello" }"#).unwrap(),
            log
        );
    }
}
