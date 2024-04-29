mod app;
mod board;
mod board_components;
mod console;
mod hooks;
use app::App;
use wasm_logger;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
