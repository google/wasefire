use log::info;
use wasm_bindgen::JsCast;
use web_common::Command;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::{function_component, html, use_effect_with, Callback, Html, Properties};
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

    let onsubmit = {
        let history = history.clone();
        let console_ref = console_ref.clone();
        let on_new_console_msg = on_new_console_msg.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let input_form: HtmlInputElement =
                console_ref.get().unwrap().value_of().dyn_into().unwrap();
            let value = input_form.value();
            info!("sending console message: {value}");
            history.push(format!("[send]: {}", value));
            on_new_console_msg.emit(value);
            input_form.set_value("");
        })
    };

    {
        let history = history.clone();
        let command_state = command_state.clone();
        use_effect_with(command_state, move |command_state| {
            if let Some(command) = &**command_state {
                info!("Command: {:?} ", command);
                if let Command::Log { message } = command {
                    history.push(format!("[recv]: {}", message));
                }
            }
            || ()
        });
    }

    html! {
        <>
            <p>
                <b>{ "Log history: " }</b>
            </p>
            {
                for history.current().iter().map(|message| {
                    html! {
                        <p>{ message }</p>
                    }
                })
            }
            <form onsubmit={onsubmit}>
                <input ref={console_ref} type="text" id="consolein" />
                <input  type="submit" value="Send"/>
            </form>
        </>
    }
}
