use yew::prelude::*;

use crate::board::Board;
use crate::console::Console;
use crate::hooks::use_runner_connection::use_runner_connection;

#[function_component(App)]
pub fn app() -> Html {
    let runner_connection = use_runner_connection(String::from("ws://127.0.0.1:5000/board"));
    let runner_con_msg = runner_connection.clone();
    let on_new_console_msg: Callback<String> = Callback::from(move |msg: String| {
         runner_con_msg.send_console_event(msg);
    });

    html! {
        <main>
            //<Board />
            <Console id ={0} command_state={runner_connection.command_state} on_new_console_msg={on_new_console_msg}/>
        </main>
    }
}
