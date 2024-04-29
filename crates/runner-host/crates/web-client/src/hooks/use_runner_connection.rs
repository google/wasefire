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
        let command = Command::Input { message: console_msg };
        self.ws.send(serde_json::to_string(&command).unwrap());
    }
}

#[hook]
pub fn use_runner_connection(backend_address: String) -> UseRunnerConnectionHandle {
    let ws = use_websocket(backend_address.clone());
    let command_state = use_state_eq(|| None);

    {
        let command_state = command_state.clone();
        let ws = ws.clone();
        let new_ws = ws.clone();
        // Receive message by depending on `ws.message`.
        use_effect_with(ws.message, move |message| {
            if let Some(message) = &**message {
                info!("Message: {message}");
                match serde_json::from_str::<Command>(message) {
                    Ok(command) => {
                        if command == Command::Connected {
                            info!("Connected to runner");
                            new_ws.send(serde_json::to_string(&Event::BoardReady).unwrap())
                        }
                        command_state.set(Some(command));
                    }
                    Err(err) => warn!("Error parsing message: {err}"),
                }
            }
            || ()
        });
    }
    return UseRunnerConnectionHandle { ws, command_state };
}
